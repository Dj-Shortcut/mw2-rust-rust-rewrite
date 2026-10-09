using System;
using System.Collections.Generic;
using System.Threading;
using UnityEngine;
using UnityEngine.SceneManagement;

namespace Shortcut.RustMod
{
    public sealed class RustSkateProbeSet
    {
        public const int ProbeLayer = 2;
        public static readonly Vector3 Parking = new Vector3(0, -10000, 0);
        private readonly int serverThread = Thread.CurrentThread.ManagedThreadId;
        private readonly OwnedProbe mounted = new OwnedProbe(SkateMotion.MountedHull);
        private readonly OwnedProbe standing = new OwnedProbe(SkateMotion.StandingHull);
        private readonly List<Component> components = new List<Component>(4);
        private int closed;
        private bool ready;
        private bool working;
        private bool allocationPending;
        private bool allocationUncertain;

        private RustSkateProbeSet() { }

        public static bool TryCreate(out RustSkateProbeSet probes, out string error)
        {
            // Return the cleanup record before any native allocation can partly succeed.
            probes = new RustSkateProbeSet();
            try
            {
                int movementMask = Rust.Layers.Server.PlayerMovement;
                if ((movementMask & (1 << ProbeLayer)) != 0 ||
                    Parking.y + (float)SkateMotion.MountedHull.HalfExtents.Y >= -4096 ||
                    Parking.y + (float)SkateMotion.StandingHull.HalfExtents.Y >= -4096)
                {
                    probes.Close();
                    return Fail("Probe parking or layer is unsafe.", out error);
                }
                probes.CreateProbe(probes.mounted, "Shortcut Skate Mounted Probe");
                probes.CreateProbe(probes.standing, "Shortcut Skate Standing Probe");
                probes.ready = true;
                if (!probes.IsCurrent)
                    return Fail("Owned skate probes did not become ready.", out error);
                error = null;
                return true;
            }
            catch (Exception)
            {
                if (probes.allocationPending) probes.allocationUncertain = true;
                probes.Close();
                return Fail("Owned skate probe creation failed; cleanup is required.", out error);
            }
        }

        public BoxCollider Mounted { get { return mounted.Collider; } }
        public BoxCollider Standing { get { return standing.Collider; } }

        public bool IsCurrent
        {
            get
            {
                if (!OnServerThread || IsClosed || !ready) return false;
                if (working) { Close(); return false; }
                working = true;
                try
                {
                    if (!ValidateProbes(false, false) || IsClosed) { Close(); return false; }
                    return true;
                }
                catch (Exception) { Close(); return false; }
                finally { working = false; }
            }
        }

        public void Close() { Interlocked.Exchange(ref closed, 1); }

        public bool TryPenetration(bool mounted, Vector3 position, Collider other,
                                   Vector3 otherPosition, Quaternion otherRotation,
                                   out bool penetrates, out Vector3 direction, out float depth)
        {
            penetrates = false;
            direction = Vector3.zero;
            depth = 0;
            if (!OnServerThread || IsClosed || !ready) return false;
            if (working) { Close(); return false; }
            working = true;
            BoxCollider selected = null;
            bool activationAttempted = false;
            bool measured = false;
            bool nativePenetrates = false;
            Vector3 nativeDirection = Vector3.zero;
            float nativeDepth = 0;
            try
            {
                if (!ValidateProbes(false, false)) { Close(); return false; }
                if (!QueryPosition(position) || !Finite(otherPosition) || !UnitRotation(otherRotation) ||
                    !ValidOther(other)) return false;
                if (IsClosed) return false;
                selected = mounted ? this.mounted.Collider : standing.Collider;
                // Nothing may yield or invoke a host callback while this parked actor is enabled.
                activationAttempted = true;
                selected.enabled = true;
                if (IsClosed || !ValidateProbes(mounted, !mounted) || !ValidOther(other))
                { Close(); return false; }
                nativePenetrates = Physics.ComputePenetration(selected, position, Quaternion.identity,
                    other, otherPosition, otherRotation, out nativeDirection, out nativeDepth);
                if (!ValidOther(other) || (nativePenetrates &&
                    (!Finite(nativeDepth) || nativeDepth < 0 || !UnitDirection(nativeDirection))))
                { Close(); return false; }
                measured = true;
            }
            catch (Exception) { Close(); measured = false; }
            finally
            {
                if (activationAttempted)
                {
                    try
                    {
                        selected.enabled = false;
                        if (!ValidateProbes(false, false)) { Close(); measured = false; }
                    }
                    catch (Exception) { Close(); measured = false; }
                }
                working = false;
            }
            if (!measured || IsClosed) return false;
            penetrates = nativePenetrates;
            // Unity leaves these undefined on a non-penetrating result.
            if (nativePenetrates) { direction = nativeDirection; depth = nativeDepth; }
            return true;
        }

        public bool TryBeginCleanup(out string error)
        {
            Close();
            if (!OnServerThread) return Fail("Probe cleanup is not on its server thread.", out error);
            if (working) return Fail("Probe operation is still in progress.", out error);
            working = true;
            try
            {
                bool cleanMounted = TryDestroy(mounted);
                bool cleanStanding = TryDestroy(standing);
                if (!cleanMounted || !cleanStanding)
                    return Fail("Owned probe destruction could not be submitted safely.", out error);
                if (allocationUncertain)
                    return Fail("A probe allocation has an unknown cleanup outcome.", out error);
                error = null;
                return true;
            }
            finally { working = false; }
        }

        public bool IsCleanupComplete
        {
            get
            {
                if (!OnServerThread || !IsClosed || working || allocationUncertain) return false;
                working = true;
                try { return mounted.Object == null && standing.Object == null; }
                catch (Exception) { return false; }
                finally { working = false; }
            }
        }

        private bool OnServerThread { get { return Thread.CurrentThread.ManagedThreadId == serverThread; } }
        private bool IsClosed { get { return Volatile.Read(ref closed) != 0; } }

        private void CreateProbe(OwnedProbe probe, string name)
        {
            allocationPending = true;
            probe.Object = new GameObject(name);
            allocationPending = false;
            probe.ObjectKey = probe.Object.GetInstanceID();
            probe.Transform = probe.Object.transform;
            probe.TransformKey = probe.Transform.GetInstanceID();
            probe.Scene = probe.Object.scene;
            probe.PhysicsScene = probe.Scene.GetPhysicsScene();
            if (probe.ObjectKey == 0 || probe.TransformKey == 0 || !probe.Scene.IsValid() ||
                !probe.Scene.isLoaded || !probe.PhysicsScene.IsValid() ||
                !probe.PhysicsScene.Equals(Physics.defaultPhysicsScene))
                throw new InvalidOperationException("Invalid probe allocation.");
            probe.Object.SetActive(false);
            if (probe.Object.activeSelf || probe.Object.activeInHierarchy)
                throw new InvalidOperationException("Probe did not become inactive.");
            probe.Object.layer = ProbeLayer;
            probe.Transform.localScale = Vector3.one;
            probe.Transform.rotation = Quaternion.identity;
            probe.Transform.position = Parking;
            probe.Collider = probe.Object.AddComponent<BoxCollider>();
            probe.Collider.enabled = false;
            probe.ColliderKey = probe.Collider.GetInstanceID();
            probe.Collider.isTrigger = false;
            probe.Collider.center = Vector3.zero;
            probe.Collider.size = probe.Size;
            if (probe.Collider.enabled || probe.ColliderKey == 0)
                throw new InvalidOperationException("Probe did not become disabled.");
            probe.Object.SetActive(true);
            probe.Initialized = true;
        }

        private bool ValidateProbes(bool mountedEnabled, bool standingEnabled)
        {
            return ready && !allocationUncertain &&
                (Rust.Layers.Server.PlayerMovement & (1 << ProbeLayer)) == 0 &&
                ValidateProbe(mounted, mountedEnabled) && ValidateProbe(standing, standingEnabled);
        }

        private bool ValidateProbe(OwnedProbe probe, bool enabled)
        {
            GameObject gameObject = probe.Object;
            Transform transform = probe.Transform;
            BoxCollider collider = probe.Collider;
            if (!probe.Initialized || gameObject == null || transform == null || collider == null ||
                gameObject.GetInstanceID() != probe.ObjectKey || transform.GetInstanceID() != probe.TransformKey ||
                collider.GetInstanceID() != probe.ColliderKey ||
                !ReferenceEquals(gameObject.transform, transform) || !ReferenceEquals(collider.transform, transform) ||
                !ReferenceEquals(collider.gameObject, gameObject) || !gameObject.activeSelf ||
                !gameObject.activeInHierarchy || gameObject.layer != ProbeLayer ||
                gameObject.scene != probe.Scene || !probe.Scene.IsValid() || !probe.Scene.isLoaded ||
                !probe.PhysicsScene.IsValid() || !probe.Scene.GetPhysicsScene().Equals(probe.PhysicsScene) ||
                !probe.PhysicsScene.Equals(Physics.defaultPhysicsScene) ||
                !ReferenceEquals(transform.parent, null) || transform.childCount != 0 ||
                !Same(transform.position, Parking) || !Same(transform.localPosition, Parking) ||
                !Same(transform.localScale, Vector3.one) || !Same(transform.lossyScale, Vector3.one) ||
                !Same(transform.rotation, Quaternion.identity) || !Same(transform.localRotation, Quaternion.identity) ||
                collider.enabled != enabled || collider.isTrigger || !Same(collider.center, Vector3.zero) ||
                !Same(collider.size, probe.Size)) return false;
            components.Clear();
            gameObject.GetComponents<Component>(components);
            return components.Count == 2 &&
                ((ReferenceEquals(components[0], transform) && ReferenceEquals(components[1], collider)) ||
                 (ReferenceEquals(components[1], transform) && ReferenceEquals(components[0], collider)));
        }

        private static bool ValidOther(Collider other)
        {
            return other != null && other.enabled && !other.isTrigger && other.gameObject != null &&
                other.gameObject.activeInHierarchy && other.transform != null;
        }

        private static bool TryDestroy(OwnedProbe probe)
        {
            try
            {
                GameObject gameObject = probe.Object;
                if (gameObject == null) return true;
                if (probe.ObjectKey != 0 && gameObject.GetInstanceID() != probe.ObjectKey) return false;
                // Destroy also removes descendants; unknown children remain a host cleanup obligation.
                Transform transform = gameObject.transform;
                if (transform == null || transform.childCount != 0) return false;
                if (!probe.DestroySubmitted)
                {
                    UnityEngine.Object.Destroy(gameObject);
                    probe.DestroySubmitted = true;
                }
                return true;
            }
            catch (Exception) { return false; }
        }

        private sealed class OwnedProbe
        {
            public readonly Vector3 Size;
            public GameObject Object;
            public Transform Transform;
            public BoxCollider Collider;
            public Scene Scene;
            public PhysicsScene PhysicsScene;
            public int ObjectKey;
            public int TransformKey;
            public int ColliderKey;
            public bool Initialized;
            public bool DestroySubmitted;

            public OwnedProbe(SkateHull hull)
            {
                Size = new Vector3((float)(hull.HalfExtents.X * 2),
                    (float)(hull.HalfExtents.Y * 2), (float)(hull.HalfExtents.Z * 2));
            }
        }

        private static bool QueryPosition(Vector3 value)
        { return Finite(value) && Math.Abs(value.x) <= 4096 && Math.Abs(value.y) <= 4096 && Math.Abs(value.z) <= 4096; }
        private static bool UnitRotation(Quaternion value)
        {
            if (!Finite(value.x) || !Finite(value.y) || !Finite(value.z) || !Finite(value.w)) return false;
            double squared = (double)value.x * value.x + (double)value.y * value.y +
                (double)value.z * value.z + (double)value.w * value.w;
            return Math.Abs(squared - 1) <= 0.001;
        }
        private static bool UnitDirection(Vector3 value)
        {
            if (!Finite(value)) return false;
            double squared = (double)value.x * value.x + (double)value.y * value.y + (double)value.z * value.z;
            return Math.Abs(squared - 1) <= 0.001;
        }
        private static bool Finite(Vector3 value) { return Finite(value.x) && Finite(value.y) && Finite(value.z); }
        private static bool Finite(float value) { return !float.IsNaN(value) && !float.IsInfinity(value); }
        private static bool Same(Vector3 a, Vector3 b) { return a.x == b.x && a.y == b.y && a.z == b.z; }
        private static bool Same(Quaternion a, Quaternion b)
        { return a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w; }
        private static bool Fail(string message, out string error) { error = message; return false; }
    }
}
