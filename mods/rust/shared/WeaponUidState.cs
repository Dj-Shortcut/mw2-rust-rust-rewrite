using System;
using System.Collections.Generic;

namespace Shortcut.RustMod
{
    public struct WeaponUidBinding
    {
        public readonly ulong Uid;
        public readonly GunState State;

        public WeaponUidBinding(ulong uid, GunState state)
        { Uid = uid; State = state; }
    }

    public struct WeaponAmmoObservation
    {
        public readonly ulong Uid;
        public readonly int Magazine;
        public readonly int Reserve;

        public WeaponAmmoObservation(ulong uid, int magazine, int reserve)
        { Uid = uid; Magazine = magazine; Reserve = reserve; }
    }

    public sealed class WeaponUidSnapshot
    {
        private readonly Dictionary<ulong, GunState> states;

        public Guid Lease { get; private set; }
        public long Tick { get; private set; }
        public ulong ActiveUid { get; private set; }
        public int Count { get { return states.Count; } }

        internal WeaponUidSnapshot(Guid lease, long tick, ulong activeUid,
            Dictionary<ulong, GunState> source)
        {
            Lease = lease; Tick = tick; ActiveUid = activeUid;
            states = new Dictionary<ulong, GunState>(source);
        }

        public bool TryGetState(ulong uid, out GunState state)
        { return states.TryGetValue(uid, out state); }

        internal Dictionary<ulong, GunState> CopyStates()
        { return new Dictionary<ulong, GunState>(states); }
    }

    public sealed class WeaponUidCandidate
    {
        internal readonly WeaponUidStateSet Owner;
        internal readonly WeaponUidSnapshot Original;
        internal readonly object Epoch;
        private readonly GunEvents events;
        private readonly ShotIntent shot;
        private readonly GunHandling handling;

        public WeaponUidSnapshot Planned { get; private set; }
        public WeaponUidSnapshot Fallback { get; private set; }

        internal WeaponUidCandidate(WeaponUidStateSet owner, WeaponUidSnapshot original,
            object epoch, WeaponUidSnapshot planned, WeaponUidSnapshot fallback,
            GunEvents events, ShotIntent shot, GunHandling handling)
        {
            Owner = owner; Original = original; Epoch = epoch;
            Planned = planned; Fallback = fallback;
            this.events = events; this.shot = shot; this.handling = handling;
        }

        public bool TryGetActiveResult(out GunEvents gunEvents, out ShotIntent shotIntent,
            out GunHandling gunHandling)
        {
            gunEvents = GunEvents.None;
            shotIntent = default(ShotIntent);
            gunHandling = default(GunHandling);
            if (Planned.ActiveUid == 0) return false;
            gunEvents = events; shotIntent = shot; gunHandling = handling;
            return true;
        }
    }

    public sealed class WeaponUidStateSet
    {
        public const int MaximumWeapons = 64;

        private readonly object sync = new object();
        private WeaponUidSnapshot current;
        private WeaponUidCandidate pending;
        private object epoch = new object();
        private bool open = true;

        private WeaponUidStateSet(Guid lease, long hostTick, Dictionary<ulong, GunState> states)
        { current = new WeaponUidSnapshot(lease, hostTick, 0, states); }

        public WeaponUidSnapshot Current { get { lock (sync) return current; } }
        public bool IsOpen { get { lock (sync) return open; } }

        public static bool TryCreate(Guid lease, long hostTick, WeaponUidBinding[] bindings,
            out WeaponUidStateSet set, out string error)
        {
            set = null;
            error = "Invalid weapon UID initialization.";
            if (lease == Guid.Empty || !ValidTick(hostTick) || bindings == null ||
                bindings.Length > MaximumWeapons) return false;
            var copied = (WeaponUidBinding[])bindings.Clone();
            var states = new Dictionary<ulong, GunState>(copied.Length);
            foreach (var binding in copied)
            {
                if (binding.Uid == 0 || binding.State == null || binding.State.Tick != hostTick ||
                    states.ContainsKey(binding.Uid)) return false;
                GunState checkedState;
                string ammoError;
                if (!Gunplay.TryReconcileAmmo(binding.State, binding.State.Magazine,
                    binding.State.Reserve, out checkedState, out ammoError)) return false;
                states.Add(binding.Uid, checkedState);
            }
            set = new WeaponUidStateSet(lease, hostTick, states);
            error = null;
            return true;
        }

        public bool TryGetState(ulong uid, out GunState state)
        { lock (sync) return current.TryGetState(uid, out state); }

        public bool TryPrepare(long hostTick, ulong activeUid, WeaponAmmoObservation[] observations,
            GunInput input, GunPose pose, out WeaponUidCandidate candidate, out string error)
        {
            lock (sync)
            {
                candidate = null;
                error = "Invalid weapon UID preparation.";
                if (!open || pending != null || current.Tick >= Gunplay.MaximumTick ||
                    !ValidTick(hostTick) || hostTick != current.Tick + 1 ||
                    pose.Mode < GunUseMode.Walking || pose.Mode > GunUseMode.Unavailable ||
                    observations == null || observations.Length != current.Count) return false;
                var baseline = current.CopyStates();
                if (activeUid != 0 && !baseline.ContainsKey(activeUid)) return false;
                var copied = (WeaponAmmoObservation[])observations.Clone();
                var seen = new Dictionary<ulong, bool>(copied.Length);
                string coreError;
                foreach (var observation in copied)
                {
                    GunState state;
                    if (!baseline.TryGetValue(observation.Uid, out state) ||
                        seen.ContainsKey(observation.Uid)) return false;
                    GunState reconciled;
                    if (!Gunplay.TryReconcileAmmo(state, observation.Magazine, observation.Reserve,
                        out reconciled, out coreError))
                    { error = coreError; return false; }
                    baseline[observation.Uid] = reconciled;
                    seen.Add(observation.Uid, true);
                }
                if (activeUid != 0 && activeUid != current.ActiveUid)
                {
                    GunState equipped;
                    if (!Gunplay.TryBeginEquip(baseline[activeUid], out equipped, out coreError))
                    { error = coreError; return false; }
                    baseline[activeUid] = equipped;
                }
                var planned = new Dictionary<ulong, GunState>(baseline.Count);
                var fallback = new Dictionary<ulong, GunState>(baseline.Count);
                var unavailable = new GunPose(GunUseMode.Unavailable, pose.Moving, pose.Airborne);
                GunEvents selectedEvents = GunEvents.None;
                ShotIntent selectedShot = default(ShotIntent);
                GunHandling selectedHandling = default(GunHandling);
                foreach (var entry in baseline)
                {
                    bool selected = entry.Key == activeUid;
                    GunState next;
                    GunEvents events;
                    ShotIntent shot;
                    GunHandling handling;
                    if (!Gunplay.TryTick(entry.Value, hostTick,
                        selected ? input : default(GunInput), selected ? pose : unavailable,
                        out next, out events, out shot, out handling, out coreError))
                    { error = coreError; return false; }
                    planned.Add(entry.Key, next);
                    if (selected)
                    {
                        selectedEvents = events; selectedShot = shot; selectedHandling = handling;
                    }
                    if (!Gunplay.TryTick(entry.Value, hostTick, default(GunInput), unavailable,
                        out next, out events, out shot, out handling, out coreError))
                    { error = coreError; return false; }
                    fallback.Add(entry.Key, next);
                }
                candidate = new WeaponUidCandidate(this, current, epoch,
                    new WeaponUidSnapshot(current.Lease, hostTick, activeUid, planned),
                    new WeaponUidSnapshot(current.Lease, hostTick, activeUid, fallback),
                    selectedEvents, selectedShot, selectedHandling);
                pending = candidate;
                error = null;
                return true;
            }
        }

        public bool TryPublishApplied(WeaponUidCandidate candidate, out string error)
        { return TryPublish(candidate, true, out error); }

        public bool TryPublishRejectedNoEffects(WeaponUidCandidate candidate, out string error)
        { return TryPublish(candidate, false, out error); }

        public void Invalidate()
        {
            lock (sync)
            {
                if (!open) return;
                open = false;
                pending = null;
                epoch = new object();
            }
        }

        private bool TryPublish(WeaponUidCandidate candidate, bool applied, out string error)
        {
            lock (sync)
            {
                error = "Invalid weapon UID publication.";
                if (!open || candidate == null || !ReferenceEquals(pending, candidate) ||
                    !ReferenceEquals(candidate.Owner, this) ||
                    !ReferenceEquals(candidate.Original, current) ||
                    !ReferenceEquals(candidate.Epoch, epoch)) return false;
                current = applied ? candidate.Planned : candidate.Fallback;
                pending = null;
                epoch = new object();
                error = null;
                return true;
            }
        }

        private static bool ValidTick(long tick)
        { return tick >= 0 && tick <= Gunplay.MaximumTick; }
    }
}
