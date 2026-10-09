using System;
using System.Threading;
using UnityEngine;

namespace Shortcut.RustMod
{
    // The host creates this binding on the server main thread after synchronizing physics.
    // The host owns the board and factory-owned parked probes, and revokes before cleanup.
    public sealed class RustSkateWorld : ISkateWorld
    {
        private const int MaximumHierarchyDepth = 128;
        private readonly BasePlayer rider;
        private readonly BaseEntity board;
        private readonly Guid boardLease;
        private readonly ulong playerId;
        private readonly BoxCollider mountedProbe;
        private readonly BoxCollider standingProbe;
        private readonly RustSkateProbeSet probeSet;
        private readonly SkateRailBinding railBinding;
        private readonly IRustSkateRailLease railLease;
        private readonly int serverThread;
        private readonly Collider[] overlaps = new Collider[SkateWorldQuery.Capacity];
        private readonly RaycastHit[] casts = new RaycastHit[SkateWorldQuery.Capacity];
        private readonly SkateWorldContact[] contacts = new SkateWorldContact[SkateWorldQuery.Capacity];
        private int closed;
        private bool querying;

        public RustSkateWorld(BasePlayer rider, BaseEntity board, Guid boardLease,
                              BoxCollider mountedProbe, BoxCollider standingProbe)
            : this(rider, board, boardLease, mountedProbe, standingProbe, SkateRailBinding.Empty)
        { }

        public RustSkateWorld(BasePlayer rider, BaseEntity board, Guid boardLease,
                              BoxCollider mountedProbe, BoxCollider standingProbe,
                              SkateRailBinding railBinding)
            : this(rider, board, boardLease, mountedProbe, standingProbe, railBinding, null)
        { }

        public RustSkateWorld(BasePlayer rider, BaseEntity board, Guid boardLease,
                              BoxCollider mountedProbe, BoxCollider standingProbe,
                              SkateRailBinding railBinding, IRustSkateRailLease railLease)
            : this(rider, board, boardLease, mountedProbe, standingProbe, railBinding, railLease, null)
        { }

        private RustSkateWorld(BasePlayer rider, BaseEntity board, Guid boardLease,
                               BoxCollider mountedProbe, BoxCollider standingProbe,
                               SkateRailBinding railBinding, IRustSkateRailLease railLease,
                               RustSkateProbeSet probeSet)
        {
            if (railBinding == null) throw new ArgumentNullException(nameof(railBinding));
            if (railLease != null && !ReferenceEquals(railLease.Binding, railBinding))
                throw new ArgumentException("Rail lease uses a different catalog.", nameof(railLease));
            this.rider = rider;
            this.board = board;
            this.boardLease = boardLease;
            playerId = ReferenceEquals(rider, null) ? 0 : rider.userID;
            this.mountedProbe = mountedProbe;
            this.standingProbe = standingProbe;
            this.probeSet = probeSet;
            this.railBinding = railBinding;
            this.railLease = railLease;
            serverThread = Thread.CurrentThread.ManagedThreadId;
        }

        public static RustSkateWorld WithProbes(BasePlayer rider, BaseEntity board, Guid boardLease,
                                                RustSkateProbeSet probes, SkateRailBinding railBinding,
                                                IRustSkateRailLease railLease)
        {
            if (probes == null) throw new ArgumentNullException(nameof(probes));
            if (!probes.IsCurrent) throw new ArgumentException("Skate probes are not current.", nameof(probes));
            return new RustSkateWorld(rider, board, boardLease, probes.Mounted, probes.Standing,
                                      railBinding, railLease, probes);
        }

        public SkateRailSet Rails { get { return railBinding.Rails; } }

        // Revocation is idempotent and safe without accessing a Unity object.
        // A lease is host fencing metadata; it does not authenticate a rider.
        public void Close() { Interlocked.Exchange(ref closed, 1); }

        public bool TrySweep(SkateVector start, SkateVector end, SkateHull hull, out SkateHit hit)
        {
            hit = SkateHit.Miss;
            if (!TryEnter()) return false;
            try
            {
                SkateWorldSweep query;
                Transform riderRoot, boardRoot;
                if (!SkateWorldQuery.TryPrepare(start, end, hull, out query) ||
                    !TryBindings(out riderRoot, out boardRoot)) return false;
                bool blocked;
                if (!TryOverlap(query, riderRoot, boardRoot, out blocked)) return false;
                SkateHit result = SkateHit.Miss;
                if (blocked)
                    result = new SkateHit(true, true, 0, new SkateVector(0, 1, 0), 0);
                else if (query.Distance > 0)
                {
                    if (IsClosed) return false;
                    Vector3 center = ToUnity(query.Center);
                    int count = Physics.BoxCastNonAlloc(center, ToUnity(query.HalfExtents),
                        ToUnity(query.Direction), casts, Quaternion.identity, (float)query.Distance,
                        Rust.Layers.Server.PlayerMovement, QueryTriggerInteraction.Ignore);
                    // A full buffer may hide a closer blocker, including after self filtering.
                    if (count < 0 || count >= casts.Length) return false;
                    for (int i = 0; i < count; ++i)
                    {
                        if (IsClosed) return false;
                        RaycastHit nativeHit = casts[i];
                        if (!Finite(nativeHit.point)) return false;
                        Collider collider = nativeHit.collider;
                        Transform colliderTransform;
                        bool self;
                        if (!TryCollider(collider, out colliderTransform) ||
                            !TrySelf(colliderTransform, riderRoot, boardRoot, out self)) return false;
                        if (!self && !GamePhysics.Verify(nativeHit, center, rider)) return false;
                        long railId = self ? 0 : railBinding.Resolve(collider.GetInstanceID());
                        contacts[i] = new SkateWorldContact(nativeHit.distance,
                            ToSkate(nativeHit.normal), self, railId);
                    }
                    if (!SkateWorldQuery.TrySelect(query, contacts, count, out result)) return false;
                }
                // This is the result publication boundary against concurrent revocation.
                if (!RailCurrent || !ProbesCurrent) return false;
                hit = result;
                return true;
            }
            catch (Exception) { return false; }
            finally { querying = false; }
        }

        public bool TryClear(SkateVector position, SkateHull hull, out bool clear)
        {
            clear = false;
            if (!TryEnter()) return false;
            try
            {
                SkateWorldSweep query;
                Transform riderRoot, boardRoot;
                if (!SkateWorldQuery.TryPrepare(position, position, hull, out query) ||
                    !TryBindings(out riderRoot, out boardRoot)) return false;
                bool blocked;
                if (!TryOverlap(query, riderRoot, boardRoot, out blocked) || !RailCurrent || !ProbesCurrent) return false;
                clear = !blocked;
                return true;
            }
            catch (Exception) { return false; }
            finally { querying = false; }
        }

        private bool IsClosed { get { return Volatile.Read(ref closed) != 0; } }

        private bool RailCurrent
        {
            get
            {
                if (IsClosed) return false;
                try
                {
                    if (railLease != null && !railLease.IsCurrent)
                    {
                        Close();
                        return false;
                    }
                }
                catch (Exception)
                {
                    Close();
                    return false;
                }
                return !IsClosed;
            }
        }

        private bool ProbesCurrent
        {
            get
            {
                if (IsClosed) return false;
                try
                {
                    if (probeSet == null || !probeSet.IsCurrent)
                    {
                        Close();
                        return false;
                    }
                }
                catch (Exception)
                {
                    Close();
                    return false;
                }
                return !IsClosed;
            }
        }

        private bool TryEnter()
        {
            // Check managed guards before touching any Unity object.
            if (Thread.CurrentThread.ManagedThreadId != serverThread || IsClosed || querying) return false;
            querying = true;
            return true;
        }

        private bool TryBindings(out Transform riderRoot, out Transform boardRoot)
        {
            riderRoot = null;
            boardRoot = null;
            if (!RailCurrent || playerId == 0 || boardLease == Guid.Empty || rider == null || board == null ||
                rider.IsDestroyed || board.IsDestroyed || rider.userID != playerId || board.OwnerID != playerId ||
                !rider.IsConnected || !rider.IsAlive()) return false;
            if (!ProbesCurrent || mountedProbe != probeSet.Mounted ||
                standingProbe != probeSet.Standing || mountedProbe == standingProbe) return false;
            riderRoot = rider.transform;
            boardRoot = board.transform;
            return riderRoot != null && boardRoot != null &&
                ValidProbe(mountedProbe, SkateMotion.MountedHull) &&
                ValidProbe(standingProbe, SkateMotion.StandingHull);
        }

        private static bool ValidProbe(BoxCollider probe, SkateHull hull)
        {
            if (probe == null || probe.enabled || probe.gameObject == null || !probe.gameObject.activeInHierarchy)
                return false;
            Transform transform = probe.transform;
            if (transform == null || transform.parent != null) return false;
            Vector3 scale = transform.lossyScale;
            Vector3 size = probe.size;
            Vector3 center = probe.center;
            Quaternion rotation = transform.rotation;
            return scale.x == 1 && scale.y == 1 && scale.z == 1 &&
                size.x == (float)(hull.HalfExtents.X * 2) &&
                size.y == (float)(hull.HalfExtents.Y * 2) &&
                size.z == (float)(hull.HalfExtents.Z * 2) &&
                center.x == 0 && center.y == 0 && center.z == 0 &&
                rotation.x == 0 && rotation.y == 0 && rotation.z == 0 && Math.Abs(rotation.w) == 1;
        }

        private bool TryOverlap(SkateWorldSweep query, Transform riderRoot, Transform boardRoot, out bool blocked)
        {
            blocked = false;
            if (IsClosed) return false;
            Vector3 center = ToUnity(query.Center);
            int count = Physics.OverlapBoxNonAlloc(center, ToUnity(query.HalfExtents), overlaps,
                Quaternion.identity, Rust.Layers.Server.PlayerMovement, QueryTriggerInteraction.Ignore);
            if (count < 0 || count >= overlaps.Length) return false;
            for (int i = 0; i < count; ++i)
            {
                if (IsClosed) return false;
                Collider collider = overlaps[i];
                Transform colliderTransform;
                bool self;
                if (!TryCollider(collider, out colliderTransform) ||
                    !TrySelf(colliderTransform, riderRoot, boardRoot, out self)) return false;
                if (self) continue;
                // Unity penetration ignores backfaces, so a concave overlap cannot certify clearance.
                MeshCollider mesh = collider as MeshCollider;
                if ((mesh != null && !mesh.convex) || !GamePhysics.Verify(collider, center, rider)) return false;
                Vector3 otherPosition = colliderTransform.position;
                Quaternion otherRotation = colliderTransform.rotation;
                if (!Finite(otherPosition) || !Finite(otherRotation)) return false;
                Vector3 direction;
                float depth;
                if (IsClosed) return false;
                bool penetrates;
                if (!probeSet.TryPenetration(query.Mounted, center, collider, otherPosition, otherRotation,
                    out penetrates, out direction, out depth))
                {
                    Close();
                    return false;
                }
                // Direction and depth are undefined when Unity reports no penetration.
                if (!penetrates) continue;
                if (!Finite(depth) || depth < 0 || !NearUnit(direction)) return false;
                if (depth > 0)
                {
                    blocked = true;
                }
            }
            return ProbesCurrent;
        }

        private static bool TryCollider(Collider collider, out Transform transform)
        {
            transform = null;
            if (collider == null || !collider.enabled || collider.isTrigger ||
                collider.gameObject == null || !collider.gameObject.activeInHierarchy) return false;
            transform = collider.transform;
            return transform != null;
        }

        private static bool TrySelf(Transform transform, Transform riderRoot, Transform boardRoot, out bool self)
        {
            self = false;
            // Compare only the exact entity transform and its descendants; never transform.root.
            for (int depth = 0; depth < MaximumHierarchyDepth; ++depth)
            {
                if (transform == null) return true;
                if (transform == riderRoot || transform == boardRoot) { self = true; return true; }
                transform = transform.parent;
            }
            return transform == null;
        }

        private static Vector3 ToUnity(SkateVector value)
        { return new Vector3((float)value.X, (float)value.Y, (float)value.Z); }

        private static SkateVector ToSkate(Vector3 value)
        { return new SkateVector(value.x, value.y, value.z); }

        private static bool NearUnit(Vector3 value)
        {
            if (!Finite(value)) return false;
            double squared = (double)value.x * value.x + (double)value.y * value.y + (double)value.z * value.z;
            return Math.Abs(squared - 1) <= 0.001;
        }

        private static bool Finite(Vector3 value)
        { return Finite(value.x) && Finite(value.y) && Finite(value.z); }

        private static bool Finite(Quaternion value)
        { return Finite(value.x) && Finite(value.y) && Finite(value.z) && Finite(value.w); }

        private static bool Finite(float value)
        { return !float.IsNaN(value) && !float.IsInfinity(value); }
    }
}
