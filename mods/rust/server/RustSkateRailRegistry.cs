using System;
using System.Collections.Generic;
using System.Threading;
using UnityEngine;

namespace Shortcut.RustMod
{
    public struct RustSkateRailRegistration
    {
        public readonly long Id;
        public readonly long Revision;
        public readonly BoxCollider Collider;

        public RustSkateRailRegistration(long id, long revision, BoxCollider collider)
        { Id = id; Revision = revision; Collider = collider; }
    }

    public sealed class RustSkateRailRegistry
    {
        private const int MaximumHierarchyDepth = 128;
        private const double AxisTolerance = 0.000001;
        private readonly int serverThread = Thread.CurrentThread.ManagedThreadId;
        private Lease current;
        private int closed;
        private bool replacing;

        public bool TryReplace(RustSkateRailRegistration[] registrations,
                               out IRustSkateRailLease lease, out string error)
        {
            lease = null;
            if (Thread.CurrentThread.ManagedThreadId != serverThread)
                return Fail("Rail registry is not on its server thread.", out error);
            if (IsClosed) return Fail("Rail registry is closed.", out error);
            if (replacing) return Fail("Rail replacement is already in progress.", out error);
            replacing = true;
            try
            {
                // Revoke before validation: a refused replacement must not retain old rails.
                Lease previous = Interlocked.Exchange(ref current, null);
                if (previous != null) previous.Expire();
                if (registrations == null || registrations.Length > SkateRailSet.MaximumCount)
                    return Fail("Invalid live rail registration count.", out error);
                var selected = (RustSkateRailRegistration[])registrations.Clone();
                var snapshots = new RailSnapshot[selected.Length];
                var rails = new SkateRail[selected.Length];
                var colliders = new SkateRailCollider[selected.Length];
                var ids = new HashSet<long>();
                var keys = new HashSet<int>();
                for (int i = 0; i < selected.Length; ++i)
                {
                    if (IsClosed) return Fail("Rail registry is closed.", out error);
                    RustSkateRailRegistration registration = selected[i];
                    if (registration.Id <= 0 || registration.Revision <= 0)
                        return Fail("Invalid rail identity.", out error);
                    if (!ids.Add(registration.Id))
                        return Fail("Duplicate rail identity.", out error);
                    if (!RailSnapshot.TryCapture(registration, out snapshots[i], out rails[i], out error))
                        return false;
                    if (!keys.Add(snapshots[i].Key))
                        return Fail("Duplicate live rail collider.", out error);
                    colliders[i] = new SkateRailCollider(snapshots[i].Key, registration.Id);
                }
                SkateRailBinding binding;
                if (!SkateRailBinding.TryCreate(rails, colliders, out binding, out error)) return false;
                var candidate = new Lease(this, binding, snapshots);
                if (!candidate.IsCurrent)
                    return Fail("Rail changed during registration.", out error);
                Interlocked.Exchange(ref current, candidate);
                // Close can race with publication without accessing the Unity scene.
                if (!candidate.IsCurrent)
                {
                    Interlocked.CompareExchange(ref current, null, candidate);
                    candidate.Expire();
                    return Fail("Rail changed during registration.", out error);
                }
                lease = candidate;
                error = null;
                return true;
            }
            catch (Exception) { return Fail("Rail registration failed.", out error); }
            finally { replacing = false; }
        }

        public void Close()
        {
            Interlocked.Exchange(ref closed, 1);
            Lease previous = Interlocked.Exchange(ref current, null);
            if (previous != null) previous.Expire();
        }

        private bool IsClosed { get { return Volatile.Read(ref closed) != 0; } }

        private sealed class Lease : IRustSkateRailLease
        {
            private readonly RustSkateRailRegistry registry;
            private readonly RailSnapshot[] snapshots;
            private readonly SkateRailBinding binding;
            private int expired;
            private bool checking;

            public Lease(RustSkateRailRegistry registry, SkateRailBinding binding, RailSnapshot[] snapshots)
            { this.registry = registry; this.binding = binding; this.snapshots = snapshots; }

            public SkateRailBinding Binding { get { return binding; } }

            public bool IsCurrent
            {
                get
                {
                    if (Thread.CurrentThread.ManagedThreadId != registry.serverThread) return false;
                    if (Volatile.Read(ref expired) != 0) return false;
                    if (registry.IsClosed || checking) { Expire(); return false; }
                    checking = true;
                    try
                    {
                        for (int i = 0; i < snapshots.Length; ++i)
                        {
                            if (Volatile.Read(ref expired) != 0 || registry.IsClosed || !snapshots[i].Matches())
                            { Expire(); return false; }
                        }
                        if (Volatile.Read(ref expired) != 0 || registry.IsClosed)
                        { Expire(); return false; }
                        return true;
                    }
                    catch (Exception) { Expire(); return false; }
                    finally { checking = false; }
                }
            }

            public void Expire() { Interlocked.Exchange(ref expired, 1); }
        }

        private sealed class RailSnapshot
        {
            private readonly BoxCollider collider;
            private readonly Vector3 centre;
            private readonly Vector3 size;
            private readonly NodeSnapshot[] hierarchy;
            public readonly int Key;

            private RailSnapshot(BoxCollider collider, int key, Vector3 centre, Vector3 size,
                                 NodeSnapshot[] hierarchy)
            {
                this.collider = collider;
                Key = key;
                this.centre = centre;
                this.size = size;
                this.hierarchy = hierarchy;
            }

            public static bool TryCapture(RustSkateRailRegistration registration, out RailSnapshot snapshot,
                                          out SkateRail rail, out string error)
            {
                snapshot = null;
                rail = default(SkateRail);
                BoxCollider collider = registration.Collider;
                if (collider == null || !collider.enabled || collider.isTrigger)
                    return Fail("Invalid live rail collider.", out error);
                int key = collider.GetInstanceID();
                Vector3 centre = collider.center;
                Vector3 size = collider.size;
                if (key == 0 || !Finite(centre) || !Positive(size))
                    return Fail("Invalid live rail box.", out error);
                var nodes = new List<NodeSnapshot>();
                Transform transform = collider.transform;
                while (!ReferenceEquals(transform, null))
                {
                    if (nodes.Count >= MaximumHierarchyDepth)
                        return Fail("Rail hierarchy is too deep.", out error);
                    NodeSnapshot node;
                    if (!NodeSnapshot.TryCapture(transform, out node))
                        return Fail("Invalid rail transform hierarchy.", out error);
                    nodes.Add(node);
                    transform = node.Parent;
                }
                if (nodes.Count == 0 || !ReferenceEquals(collider.gameObject, nodes[0].GameObject))
                    return Fail("Invalid rail transform hierarchy.", out error);
                int layer = collider.gameObject.layer;
                int movementMask = Rust.Layers.Server.PlayerMovement;
                if (layer < 0 || layer > 31 || (movementMask & (1 << layer)) == 0)
                    return Fail("Rail layer is outside player movement queries.", out error);
                Matrix4x4 matrix = nodes[0].Matrix;
                SkateVector worldCentre, halfExtents;
                double yaw;
                if (!TryGeometry(matrix, centre, size, out worldCentre, out halfExtents, out yaw))
                    return Fail("Rail transform is not a level positive box.", out error);
                if (!SkateRailBinding.TryRailFromBox(registration.Id, registration.Revision, worldCentre,
                    halfExtents, yaw, out rail, out error)) return false;
                snapshot = new RailSnapshot(collider, key, centre, size, nodes.ToArray());
                if (!snapshot.Matches()) return Fail("Rail changed during registration.", out error);
                error = null;
                return true;
            }

            public bool Matches()
            {
                if (collider == null || collider.GetInstanceID() != Key || !collider.enabled ||
                    collider.isTrigger || !Same(collider.center, centre) || !Same(collider.size, size) ||
                    !ReferenceEquals(collider.transform, hierarchy[0].Transform) ||
                    !ReferenceEquals(collider.gameObject, hierarchy[0].GameObject)) return false;
                Transform transform = hierarchy[0].Transform;
                for (int i = 0; i < hierarchy.Length; ++i)
                {
                    if (!hierarchy[i].Matches(transform)) return false;
                    transform = hierarchy[i].Parent;
                }
                return ReferenceEquals(transform, null);
            }
        }

        private sealed class NodeSnapshot
        {
            public readonly Transform Transform;
            public readonly Transform Parent;
            public readonly GameObject GameObject;
            public readonly Matrix4x4 Matrix;
            private readonly int transformKey;
            private readonly int objectKey;
            private readonly Vector3 position;
            private readonly Quaternion rotation;
            private readonly Vector3 scale;
            private readonly int layer;
            private readonly bool activeSelf;

            private NodeSnapshot(Transform transform, Transform parent, GameObject gameObject, int transformKey,
                                 int objectKey, Vector3 position, Quaternion rotation, Vector3 scale,
                                 Matrix4x4 matrix, int layer, bool activeSelf)
            {
                Transform = transform;
                Parent = parent;
                GameObject = gameObject;
                this.transformKey = transformKey;
                this.objectKey = objectKey;
                this.position = position;
                this.rotation = rotation;
                this.scale = scale;
                Matrix = matrix;
                this.layer = layer;
                this.activeSelf = activeSelf;
            }

            public static bool TryCapture(Transform transform, out NodeSnapshot snapshot)
            {
                snapshot = null;
                if (transform == null) return false;
                GameObject gameObject = transform.gameObject;
                if (gameObject == null || !gameObject.activeInHierarchy) return false;
                int transformKey = transform.GetInstanceID(), objectKey = gameObject.GetInstanceID();
                Vector3 position = transform.localPosition;
                Quaternion rotation = transform.localRotation;
                Vector3 scale = transform.localScale;
                Matrix4x4 matrix = transform.localToWorldMatrix;
                if (transformKey == 0 || objectKey == 0 || !Finite(position) || !Finite(rotation) ||
                    !Positive(scale) || !Finite(matrix)) return false;
                snapshot = new NodeSnapshot(transform, transform.parent, gameObject, transformKey, objectKey,
                    position, rotation, scale, matrix, gameObject.layer, gameObject.activeSelf);
                return true;
            }

            public bool Matches(Transform transform)
            {
                return transform != null && ReferenceEquals(transform, Transform) &&
                    transform.GetInstanceID() == transformKey && GameObject != null &&
                    ReferenceEquals(transform.gameObject, GameObject) && GameObject.GetInstanceID() == objectKey &&
                    GameObject.activeInHierarchy && GameObject.activeSelf == activeSelf && GameObject.layer == layer &&
                    ReferenceEquals(transform.parent, Parent) && Same(transform.localPosition, position) &&
                    Same(transform.localRotation, rotation) && Same(transform.localScale, scale) &&
                    Same(transform.localToWorldMatrix, Matrix);
            }
        }

        private static bool TryGeometry(Matrix4x4 matrix, Vector3 centre, Vector3 size,
                                        out SkateVector worldCentre, out SkateVector halfExtents, out double yaw)
        {
            worldCentre = default(SkateVector);
            halfExtents = default(SkateVector);
            yaw = 0;
            if (!Finite(matrix) || matrix.m30 != 0 || matrix.m31 != 0 || matrix.m32 != 0 || matrix.m33 != 1)
                return false;
            var x = new SkateVector(matrix.m00, matrix.m10, matrix.m20);
            var y = new SkateVector(matrix.m01, matrix.m11, matrix.m21);
            var z = new SkateVector(matrix.m02, matrix.m12, matrix.m22);
            double sx = x.Length, sy = y.Length, sz = z.Length;
            if (!Finite(sx) || !Finite(sy) || !Finite(sz) || sx <= 0 || sy <= 0 || sz <= 0) return false;
            x = x * (1 / sx); y = y * (1 / sy); z = z * (1 / sz);
            if (Math.Abs(SkateVector.Dot(x, y)) > AxisTolerance || Math.Abs(SkateVector.Dot(x, z)) > AxisTolerance ||
                Math.Abs(SkateVector.Dot(y, z)) > AxisTolerance || Math.Abs(x.Y) > AxisTolerance ||
                Math.Abs(z.Y) > AxisTolerance || Math.Abs(y.X) > AxisTolerance ||
                Math.Abs(y.Z) > AxisTolerance || Math.Abs(y.Y - 1) > AxisTolerance ||
                Math.Abs(SkateVector.Dot(x, Cross(y, z)) - 1) > AxisTolerance) return false;
            worldCentre = new SkateVector(
                matrix.m00 * (double)centre.x + matrix.m01 * (double)centre.y + matrix.m02 * (double)centre.z + matrix.m03,
                matrix.m10 * (double)centre.x + matrix.m11 * (double)centre.y + matrix.m12 * (double)centre.z + matrix.m13,
                matrix.m20 * (double)centre.x + matrix.m21 * (double)centre.y + matrix.m22 * (double)centre.z + matrix.m23);
            halfExtents = new SkateVector(size.x * sx * 0.5, size.y * sy * 0.5, size.z * sz * 0.5);
            yaw = Math.Atan2(z.X, z.Z) * 180 / Math.PI;
            return Finite(worldCentre.X) && Finite(worldCentre.Y) && Finite(worldCentre.Z) &&
                Math.Abs(worldCentre.X) <= 100000 && Math.Abs(worldCentre.Y) <= 100000 &&
                Math.Abs(worldCentre.Z) <= 100000;
        }

        private static SkateVector Cross(SkateVector a, SkateVector b)
        { return new SkateVector(a.Y * b.Z - a.Z * b.Y, a.Z * b.X - a.X * b.Z, a.X * b.Y - a.Y * b.X); }
        private static bool Positive(Vector3 value)
        { return Finite(value) && value.x > 0 && value.y > 0 && value.z > 0; }
        private static bool Finite(Vector3 value) { return Finite(value.x) && Finite(value.y) && Finite(value.z); }
        private static bool Finite(Quaternion value)
        { return Finite(value.x) && Finite(value.y) && Finite(value.z) && Finite(value.w); }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
        private static bool Finite(Matrix4x4 value)
        {
            return Finite(value.m00) && Finite(value.m01) && Finite(value.m02) && Finite(value.m03) &&
                Finite(value.m10) && Finite(value.m11) && Finite(value.m12) && Finite(value.m13) &&
                Finite(value.m20) && Finite(value.m21) && Finite(value.m22) && Finite(value.m23) &&
                Finite(value.m30) && Finite(value.m31) && Finite(value.m32) && Finite(value.m33);
        }
        private static bool Same(Vector3 a, Vector3 b) { return a.x == b.x && a.y == b.y && a.z == b.z; }
        private static bool Same(Quaternion a, Quaternion b)
        { return a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w; }
        private static bool Same(Matrix4x4 a, Matrix4x4 b)
        {
            return a.m00 == b.m00 && a.m01 == b.m01 && a.m02 == b.m02 && a.m03 == b.m03 &&
                a.m10 == b.m10 && a.m11 == b.m11 && a.m12 == b.m12 && a.m13 == b.m13 &&
                a.m20 == b.m20 && a.m21 == b.m21 && a.m22 == b.m22 && a.m23 == b.m23 &&
                a.m30 == b.m30 && a.m31 == b.m31 && a.m32 == b.m32 && a.m33 == b.m33;
        }
        private static bool Fail(string message, out string error) { error = message; return false; }
    }
}
