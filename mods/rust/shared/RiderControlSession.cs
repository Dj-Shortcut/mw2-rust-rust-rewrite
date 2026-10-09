using System;

namespace Shortcut.RustMod
{
    public struct RiderIdentity
    {
        public readonly ulong PlayerId;
        public readonly Guid RuntimeId;
        public readonly Guid LifeId;

        public RiderIdentity(ulong playerId, Guid runtimeId, Guid lifeId)
        { PlayerId = playerId; RuntimeId = runtimeId; LifeId = lifeId; }
    }

    // Controls contain no host clock, world, weapon selection or physical mode.
    // The adapter must supply a fresh sample for this tick, rather than queued input.
    public sealed class RiderControlRequest
    {
        public RiderIdentity Identity { get; private set; }
        public long Sequence { get; private set; }
        public GunInput Gun { get; private set; }
        public SkateInput Skate { get; private set; }

        public RiderControlRequest(RiderIdentity identity, long sequence, GunInput gun, SkateInput skate)
        { Identity = identity; Sequence = sequence; Gun = gun; Skate = skate; }
    }

    public sealed class RiderControlFrame
    {
        public RiderIdentity Identity { get; private set; }
        public long Tick { get; private set; }
        public long Sequence { get; private set; }
        public GunPose ConfirmedPose { get; private set; }
        public Guid BoardLease { get; private set; }
        public GunInput Gun { get; private set; }
        public GunPose Pose { get; private set; }
        public SkateInput Skate { get; private set; }
        public bool FireLocked { get; private set; }
        public bool ReloadLocked { get; private set; }
        public bool JumpLocked { get; private set; }

        internal RiderControlFrame(RiderIdentity identity, long tick, long sequence,
            GunPose confirmedPose, Guid boardLease, GunInput gun, GunPose pose, SkateInput skate,
            bool fireLocked, bool reloadLocked, bool jumpLocked)
        {
            Identity = identity; Tick = tick; Sequence = sequence;
            ConfirmedPose = confirmedPose; BoardLease = boardLease;
            Gun = gun; Pose = pose; Skate = skate;
            FireLocked = fireLocked; ReloadLocked = reloadLocked; JumpLocked = jumpLocked;
        }
    }

    public sealed class RiderControlCandidate
    {
        public RiderControlFrame Frame { get; private set; }
        internal readonly RiderControlSession Owner;
        internal readonly RiderControlFrame Original;
        internal readonly object Epoch;
        internal readonly bool UsesObservation;
        internal readonly RiderInputObservation Observation;
        internal readonly long ObservationFenceTick;

        internal RiderControlCandidate(RiderControlSession owner, RiderControlFrame original,
            object epoch, RiderControlFrame frame)
        { Owner = owner; Original = original; Epoch = epoch; Frame = frame; }

        internal RiderControlCandidate(RiderControlSession owner, RiderControlFrame original,
            object epoch, RiderControlFrame frame, RiderInputObservation observation, long fenceTick)
            : this(owner, original, epoch, frame)
        { UsesObservation = true; Observation = observation; ObservationFenceTick = fenceTick; }
    }

    public enum RiderCommitOutcome { Applied = 0, RejectedNoEffects = 1, UnknownPartial = 2 }

    public sealed class RiderCleanupTicket
    {
        public RiderIdentity Identity { get; private set; }
        public Guid FirstLease { get; private set; }
        public Guid SecondLease { get; private set; }
        public int LeaseCount { get; private set; }
        public bool RequiresReconciliation { get; private set; }
        internal readonly RiderControlSession Owner;

        internal RiderCleanupTicket(RiderControlSession owner, RiderIdentity identity,
            Guid committedLease, Guid pendingLease, bool requiresReconciliation)
        {
            Owner = owner; Identity = identity; RequiresReconciliation = requiresReconciliation;
            FirstLease = committedLease;
            if (FirstLease == Guid.Empty) FirstLease = pendingLease;
            else if (pendingLease != Guid.Empty && pendingLease != FirstLease) SecondLease = pendingLease;
            LeaseCount = FirstLease == Guid.Empty ? 0 : (SecondLease == Guid.Empty ? 1 : 2);
        }
    }

    // This lock protects source bookkeeping only. The adapter must also serialize
    // native effects, lifecycle invalidation and settlement under its own fence.
    // Guid identities are host-issued fences, not network authentication.
    public sealed class RiderControlSession
    {
        private readonly object sync = new object();
        private RiderControlFrame current;
        private RiderControlCandidate pending;
        private RiderCleanupTicket cleanup;
        private object epoch = new object();
        private long highWaterTick;
        private RiderInputObservation retainedObservation;
        private long observationFenceTick;
        private bool open = true;
        private bool cleanupAcknowledged;

        private RiderControlSession(RiderIdentity identity, long hostTick)
        {
            current = InitialFrame(identity, hostTick);
            highWaterTick = hostTick;
            observationFenceTick = hostTick;
        }

        // Current is the last published frame. A pending plan and a closed session
        // do not publish their candidate state through this property.
        public RiderControlFrame Current { get { lock (sync) return current; } }
        public bool IsOpen { get { lock (sync) return open; } }
        public bool CleanupAcknowledged { get { lock (sync) return cleanupAcknowledged; } }

        public static bool TryCreate(RiderIdentity identity, long hostTick,
            out RiderControlSession session, out string error)
        {
            session = null;
            error = "Invalid rider initialization.";
            if (!ValidIdentity(identity) || !ValidTick(hostTick)) return false;
            session = new RiderControlSession(identity, hostTick);
            error = null;
            return true;
        }

        public bool TryPrepare(long hostTick, GunPose confirmedPose, Guid boardLease,
            RiderControlRequest request, out RiderControlCandidate candidate, out string error)
        {
            lock (sync)
            {
                candidate = null;
                error = "Invalid rider control preparation.";
                if (!open || pending != null || current.Tick >= Gunplay.MaximumTick ||
                    !ValidTick(hostTick) || hostTick != current.Tick + 1 ||
                    !ValidPose(confirmedPose, boardLease)) return false;
                if (request != null && (!SameIdentity(request.Identity, current.Identity) ||
                    request.Sequence <= 0 || request.Sequence <= current.Sequence ||
                    !ValidAxis(request.Skate.Steer) || !ValidAxis(request.Skate.Spin) ||
                    !ValidAxis(request.Skate.Flip))) return false;

                bool reset = request == null || confirmedPose.Mode != current.ConfirmedPose.Mode ||
                    boardLease != current.BoardLease;
                bool fireLocked = reset || current.FireLocked;
                bool reloadLocked = reset || current.ReloadLocked;
                bool jumpLocked = reset || current.JumpLocked;
                long sequence = current.Sequence;
                GunInput gun = default(GunInput);
                SkateInput skate = default(SkateInput);
                GunPose pose = UnavailablePose(confirmedPose);
                if (request != null)
                {
                    sequence = request.Sequence;
                    if (!request.Gun.Fire) fireLocked = false;
                    if (!request.Gun.Reload) reloadLocked = false;
                    if (!request.Skate.Jump) jumpLocked = false;
                    pose = request.Gun.Fire && fireLocked ? UnavailablePose(confirmedPose) : confirmedPose;
                    bool walking = pose.Mode == GunUseMode.Walking;
                    // Preserve a held Fire button; Unavailable prevents firing and
                    // prevents Gunplay from accepting a fabricated Walking release.
                    gun = new GunInput(request.Gun.Fire, walking && request.Gun.Aim,
                        walking && !reloadLocked && request.Gun.Reload);
                    if (confirmedPose.Mode == GunUseMode.Skating)
                        skate = new SkateInput(request.Skate.Push, request.Skate.Brake,
                            !jumpLocked && request.Skate.Jump, request.Skate.Steer,
                            request.Skate.Spin, request.Skate.Flip);
                }
                var frame = new RiderControlFrame(current.Identity, hostTick, sequence,
                    confirmedPose, boardLease, gun, pose, skate, fireLocked, reloadLocked, jumpLocked);
                candidate = new RiderControlCandidate(this, current, epoch, frame);
                pending = candidate;
                highWaterTick = hostTick;
                error = null;
                return true;
            }
        }

        public bool TryPrepareObserved(long hostTick, GunPose confirmedPose, Guid boardLease,
            RiderInputAdmission admission, RiderInputObservation observation,
            out RiderControlCandidate candidate, out string error)
        {
            lock (sync)
            {
                candidate = null;
                error = "Invalid observed rider control preparation.";
                if (!open || pending != null || current.Tick >= Gunplay.MaximumTick ||
                    !ValidTick(hostTick) || hostTick != current.Tick + 1 ||
                    !ValidPose(confirmedPose, boardLease) ||
                    admission < RiderInputAdmission.NewObservation ||
                    admission > RiderInputAdmission.Unavailable) return false;
                bool fresh = admission == RiderInputAdmission.NewObservation;
                if (fresh != (observation != null)) return false;
                if (fresh && (observation.AdmissionTick != hostTick || !observation.IsValidAt(hostTick) ||
                    !SameIdentity(observation.Request.Identity, current.Identity) ||
                    observation.Request.Sequence <= current.Sequence)) return false;

                bool transition = confirmedPose.Mode != current.ConfirmedPose.Mode ||
                    boardLease != current.BoardLease;
                long fenceTick = transition ? hostTick : observationFenceTick;
                RiderInputObservation retained = null;
                if (fresh) retained = observation;
                else if (admission == RiderInputAdmission.NoNewObservation && !transition &&
                    retainedObservation != null && retainedObservation.IsValidAt(hostTick))
                    retained = retainedObservation;

                // A new packet covers this slot even at the previous snapshot's expiry.
                // Uncovered slots and transitions cannot borrow a previous release.
                bool reset = transition || retainedObservation == null || retained == null;
                bool fireLocked = reset || current.FireLocked;
                bool reloadLocked = reset || current.ReloadLocked;
                bool jumpLocked = reset || current.JumpLocked;
                long sequence = current.Sequence;
                GunInput gun = default(GunInput);
                SkateInput skate = default(SkateInput);
                GunPose pose = UnavailablePose(confirmedPose);
                if (retained == null) fenceTick = hostTick;
                else
                {
                    var request = retained.Request;
                    if (fresh)
                    {
                        sequence = request.Sequence;
                        if (hostTick > fenceTick)
                        {
                            if (!request.Gun.Fire) fireLocked = false;
                            if (!request.Gun.Reload) reloadLocked = false;
                            if (!request.Skate.Jump) jumpLocked = false;
                        }
                    }
                    pose = fireLocked ? UnavailablePose(confirmedPose) : confirmedPose;
                    bool walking = pose.Mode == GunUseMode.Walking;
                    gun = new GunInput(request.Gun.Fire, walking && request.Gun.Aim,
                        walking && !reloadLocked && request.Gun.Reload);
                    if (confirmedPose.Mode == GunUseMode.Skating)
                        skate = new SkateInput(request.Skate.Push, request.Skate.Brake,
                            !jumpLocked && request.Skate.Jump, request.Skate.Steer,
                            request.Skate.Spin, request.Skate.Flip);
                }
                var frame = new RiderControlFrame(current.Identity, hostTick, sequence,
                    confirmedPose, boardLease, gun, pose, skate, fireLocked, reloadLocked, jumpLocked);
                candidate = new RiderControlCandidate(this, current, epoch, frame, retained, fenceTick);
                pending = candidate;
                highWaterTick = hostTick;
                error = null;
                return true;
            }
        }

        // A true result means this outcome was accepted, not that native work succeeded.
        // RejectedNoEffects returns a fallback for the same core tick, to apply from
        // the original/reconciled UID states without resetting ammunition or cadence.
        public bool TrySettle(RiderControlCandidate candidate, RiderCommitOutcome outcome,
            out RiderControlFrame frame, out RiderCleanupTicket ticket, out string error)
        {
            lock (sync)
            {
                frame = null; ticket = null;
                error = "Invalid rider control settlement.";
                if (!open || candidate == null || !ReferenceEquals(pending, candidate) ||
                    !ReferenceEquals(candidate.Owner, this) || !ReferenceEquals(candidate.Original, current) ||
                    !ReferenceEquals(candidate.Epoch, epoch) || outcome < RiderCommitOutcome.Applied ||
                    outcome > RiderCommitOutcome.UnknownPartial) return false;
                if (outcome == RiderCommitOutcome.UnknownPartial)
                {
                    ticket = Close(true);
                }
                else
                {
                    var plan = candidate.Frame;
                    current = outcome == RiderCommitOutcome.Applied ? plan :
                        NoActionsFrame(plan.Identity, plan.Tick, plan.Sequence, plan.ConfirmedPose, plan.BoardLease);
                    retainedObservation = outcome == RiderCommitOutcome.Applied && candidate.UsesObservation ?
                        candidate.Observation : null;
                    observationFenceTick = outcome == RiderCommitOutcome.Applied && candidate.UsesObservation ?
                        candidate.ObservationFenceTick : plan.Tick;
                    pending = null;
                    frame = current;
                }
                error = null;
                return true;
            }
        }

        public RiderCleanupTicket Invalidate()
        {
            lock (sync)
            {
                return open ? Close(false) : cleanup;
            }
        }

        // The adapter attests cleanup and, when required, observation/reconciliation
        // of possible native ammunition/entity effects. This is not native rollback.
        public bool TryAcknowledgeCleanup(RiderCleanupTicket ticket, bool nativeReconciled, out string error)
        {
            lock (sync)
            {
                error = "Invalid rider cleanup acknowledgement.";
                if (open || ticket == null || !ReferenceEquals(cleanup, ticket) ||
                    !ReferenceEquals(ticket.Owner, this) || !SameIdentity(ticket.Identity, current.Identity) ||
                    (ticket.RequiresReconciliation && !nativeReconciled)) return false;
                cleanupAcknowledged = true;
                error = null;
                return true;
            }
        }

        // Fresh life IDs must never be reused by the host. A different runtime may
        // start a new clock; within the same runtime even an invalidated plan counts.
        public bool TryRestart(RiderIdentity identity, long hostTick, out string error)
        {
            lock (sync)
            {
                error = "Invalid rider restart.";
                if (open || !cleanupAcknowledged || cleanup == null || pending != null ||
                    !ValidIdentity(identity) || !ValidTick(hostTick) ||
                    identity.PlayerId != current.Identity.PlayerId || identity.LifeId == current.Identity.LifeId ||
                    (identity.RuntimeId == current.Identity.RuntimeId && hostTick < highWaterTick)) return false;
                current = InitialFrame(identity, hostTick);
                highWaterTick = hostTick;
                retainedObservation = null;
                observationFenceTick = hostTick;
                epoch = new object();
                cleanup = null;
                cleanupAcknowledged = false;
                open = true;
                error = null;
                return true;
            }
        }

        private RiderCleanupTicket Close(bool requiresReconciliation)
        {
            Guid pendingLease = pending == null ? Guid.Empty : pending.Frame.BoardLease;
            cleanup = new RiderCleanupTicket(this, current.Identity, current.BoardLease,
                pendingLease, requiresReconciliation);
            retainedObservation = null;
            observationFenceTick = highWaterTick;
            pending = null;
            epoch = new object();
            cleanupAcknowledged = false;
            open = false;
            return cleanup;
        }

        private static RiderControlFrame InitialFrame(RiderIdentity identity, long hostTick)
        { return NoActionsFrame(identity, hostTick, 0, new GunPose(GunUseMode.Unavailable, false, false), Guid.Empty); }

        private static RiderControlFrame NoActionsFrame(RiderIdentity identity, long tick, long sequence,
            GunPose confirmedPose, Guid boardLease)
        {
            return new RiderControlFrame(identity, tick, sequence, confirmedPose, boardLease,
                default(GunInput), UnavailablePose(confirmedPose), default(SkateInput), true, true, true);
        }

        private static GunPose UnavailablePose(GunPose confirmedPose)
        { return new GunPose(GunUseMode.Unavailable, confirmedPose.Moving, confirmedPose.Airborne); }

        private static bool ValidIdentity(RiderIdentity identity)
        { return identity.PlayerId > 0 && identity.RuntimeId != Guid.Empty && identity.LifeId != Guid.Empty; }

        private static bool SameIdentity(RiderIdentity a, RiderIdentity b)
        { return a.PlayerId == b.PlayerId && a.RuntimeId == b.RuntimeId && a.LifeId == b.LifeId; }

        private static bool ValidTick(long tick) { return tick >= 0 && tick <= Gunplay.MaximumTick; }

        private static bool ValidAxis(double axis)
        { return !double.IsNaN(axis) && !double.IsInfinity(axis) && axis >= -1 && axis <= 1; }

        private static bool ValidPose(GunPose pose, Guid boardLease)
        {
            if (pose.Mode < GunUseMode.Walking || pose.Mode > GunUseMode.Unavailable) return false;
            if (pose.Mode == GunUseMode.Skating) return boardLease != Guid.Empty;
            if (pose.Mode == GunUseMode.Walking || pose.Mode == GunUseMode.Sprinting) return boardLease == Guid.Empty;
            return true;
        }
    }
}
