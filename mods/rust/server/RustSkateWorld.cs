using System;
using System.Threading;
using UnityEngine;

namespace Shortcut.RustMod
{
    // The host creates this binding on the server main thread after synchronizing physics.
    // The host owns the board and factory-owned parked probes, and revokes before cleanup.
    public sealed class RustSkateWorld : ISkateWorld
    {
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
        private readonly RustSkateCollisionScene scene;
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
            if (probeSet != null) scene = new RustSkateCollisionScene(probeSet, railBinding, railLease);
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
        public void Close()
        {
            Interlocked.Exchange(ref closed, 1);
            if (scene != null) scene.Close();
        }

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
                SkateHit result;
                if (scene == null || !scene.TrySweepFor(start, end, hull, rider,
                    riderRoot, boardRoot, out result)) return false;
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
                bool result;
                if (scene == null || !scene.TryClearFor(position, hull, rider,
                    riderRoot, boardRoot, out result) || !RailCurrent || !ProbesCurrent) return false;
                clear = result;
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

    }
}
