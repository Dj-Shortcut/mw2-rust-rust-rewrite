using System;

namespace Shortcut.RustMod
{
    public enum GunFireMode { SemiAutomatic, Automatic }
    public enum GunUseMode { Walking, Sprinting, Skating, Unavailable }

    [Flags]
    public enum GunEvents { None = 0, Fired = 1, DryFire = 2, ReloadStarted = 4, ReloadCompleted = 8, ReloadCancelled = 16 }

    public sealed class GunTuning
    {
        public GunFireMode FireMode;
        public int MagazineCapacity;
        public int FireTicks;
        public int EquipTicks;
        public int ReloadTicks;
        public int AdsTicks;
        public int RecoveryTicks;
        public double PitchKick;
        public double YawKick;
        public double PitchLimit;
        public double YawLimit;
        public double ReturnPerTick;
        public double HipSpread;
        public double AimSpread;
        public double MovingSpread;
        public double AirborneSpread;
        public double SpreadLimit;
        public double WalkScale;
        public double AimScale;
        public double SprintScale;

        public static GunTuning Rifle()
        {
            return new GunTuning {
                FireMode = GunFireMode.Automatic, MagazineCapacity = 30,
                FireTicks = 10, EquipTicks = 30, ReloadTicks = 220, AdsTicks = 20, RecoveryTicks = 15,
                PitchKick = 0.65, YawKick = 0.15, PitchLimit = 10, YawLimit = 5, ReturnPerTick = 0.03,
                HipSpread = 2.2, AimSpread = 0.2, MovingSpread = 1.2, AirborneSpread = 2, SpreadLimit = 6,
                WalkScale = 1, AimScale = 0.75, SprintScale = 1.15
            };
        }

        public static GunTuning Sidearm()
        {
            return new GunTuning {
                FireMode = GunFireMode.SemiAutomatic, MagazineCapacity = 12,
                FireTicks = 16, EquipTicks = 20, ReloadTicks = 160, AdsTicks = 12, RecoveryTicks = 10,
                PitchKick = 0.9, YawKick = 0.1, PitchLimit = 10, YawLimit = 5, ReturnPerTick = 0.04,
                HipSpread = 2, AimSpread = 0.25, MovingSpread = 1, AirborneSpread = 2, SpreadLimit = 6,
                WalkScale = 1, AimScale = 0.8, SprintScale = 1.15
            };
        }
    }

    public sealed class GunProfile
    {
        public GunFireMode FireMode { get; private set; }
        public int MagazineCapacity { get; private set; }
        public int FireTicks { get; private set; }
        public int EquipTicks { get; private set; }
        public int ReloadTicks { get; private set; }
        public int AdsTicks { get; private set; }
        public int RecoveryTicks { get; private set; }
        public double PitchKick { get; private set; }
        public double YawKick { get; private set; }
        public double PitchLimit { get; private set; }
        public double YawLimit { get; private set; }
        public double ReturnPerTick { get; private set; }
        public double HipSpread { get; private set; }
        public double AimSpread { get; private set; }
        public double MovingSpread { get; private set; }
        public double AirborneSpread { get; private set; }
        public double SpreadLimit { get; private set; }
        public double WalkScale { get; private set; }
        public double AimScale { get; private set; }
        public double SprintScale { get; private set; }

        private GunProfile(GunTuning t)
        {
            FireMode = t.FireMode; MagazineCapacity = t.MagazineCapacity;
            FireTicks = t.FireTicks; EquipTicks = t.EquipTicks; ReloadTicks = t.ReloadTicks;
            AdsTicks = t.AdsTicks; RecoveryTicks = t.RecoveryTicks;
            PitchKick = t.PitchKick; YawKick = t.YawKick; PitchLimit = t.PitchLimit; YawLimit = t.YawLimit;
            ReturnPerTick = t.ReturnPerTick; HipSpread = t.HipSpread; AimSpread = t.AimSpread;
            MovingSpread = t.MovingSpread; AirborneSpread = t.AirborneSpread; SpreadLimit = t.SpreadLimit;
            WalkScale = t.WalkScale; AimScale = t.AimScale; SprintScale = t.SprintScale;
        }

        public static bool TryCreate(GunTuning tuning, out GunProfile profile, out string error)
        {
            profile = null;
            error = "Invalid gunplay profile.";
            if (tuning == null) return false;
            var t = new GunProfile(tuning);
            if ((t.FireMode != GunFireMode.SemiAutomatic && t.FireMode != GunFireMode.Automatic) ||
                !Between(t.MagazineCapacity, 1, 100) || !Between(t.FireTicks, 4, 200) ||
                !Between(t.EquipTicks, 1, 500) || !Between(t.ReloadTicks, 1, 1000) ||
                !Between(t.AdsTicks, 1, 200) || !Between(t.RecoveryTicks, 1, 200) ||
                !Range(t.PitchLimit, 0.1, 45) || !Range(t.YawLimit, 0.1, 45) ||
                !Range(t.PitchKick, 0, t.PitchLimit) || !Range(t.YawKick, 0, t.YawLimit) ||
                !Range(t.ReturnPerTick, 0.001, 10) || !Range(t.SpreadLimit, 0.01, 45) ||
                !Range(t.HipSpread, 0, t.SpreadLimit) || !Range(t.AimSpread, 0, t.HipSpread) ||
                !Range(t.MovingSpread, 0, t.SpreadLimit) || !Range(t.AirborneSpread, 0, t.SpreadLimit) ||
                !Range(t.WalkScale, 0.1, 1.5) || !Range(t.AimScale, 0.1, t.WalkScale) ||
                !Range(t.SprintScale, t.WalkScale, 1.5)) return false;
            profile = t;
            error = null;
            return true;
        }

        private static bool Between(int v, int min, int max) { return v >= min && v <= max; }
        private static bool Range(double v, double min, double max)
        {
            return !double.IsNaN(v) && !double.IsInfinity(v) && v >= min && v <= max;
        }
    }

    public struct GunInput
    {
        public readonly bool Fire, Aim, Reload;
        public GunInput(bool fire, bool aim, bool reload) { Fire = fire; Aim = aim; Reload = reload; }
    }

    public struct GunPose
    {
        public readonly GunUseMode Mode;
        public readonly bool Moving, Airborne;
        public GunPose(GunUseMode mode, bool moving, bool airborne) { Mode = mode; Moving = moving; Airborne = airborne; }
    }

    public struct GunHandling
    {
        public readonly double Aim, Pitch, Yaw, Spread, MovementScale;
        internal GunHandling(double aim, double pitch, double yaw, double spread, double movementScale)
        {
            Aim = aim; Pitch = pitch; Yaw = yaw; Spread = spread; MovementScale = movementScale;
        }
    }

    public struct ShotIntent
    {
        public readonly long Sequence, Tick;
        public readonly double Spread, PitchKick, YawKick;
        internal ShotIntent(long sequence, long tick, double spread, double pitchKick, double yawKick)
        {
            Sequence = sequence; Tick = tick; Spread = spread; PitchKick = pitchKick; YawKick = yawKick;
        }
    }

    public sealed class GunState
    {
        public GunProfile Profile { get; private set; }
        public long Tick { get; private set; }
        public long ShotSequence { get; private set; }
        public int Magazine { get; private set; }
        public int Reserve { get; private set; }
        public int FireRemaining { get; private set; }
        public int EquipRemaining { get; private set; }
        public int ReloadRemaining { get; private set; }
        public int RecoveryRemaining { get; private set; }
        public int AimProgress { get; private set; }
        public double RecoilPitch { get; private set; }
        public double RecoilYaw { get; private set; }
        public bool RequiresRelease { get; private set; }
        public bool PreviousFire { get; private set; }
        public bool PreviousReload { get; private set; }
        public GunUseMode PreviousMode { get; private set; }

        internal GunState(GunProfile profile, long tick, long sequence, int magazine, int reserve,
            int fire, int equip, int reload, int recovery, int aim, double pitch, double yaw,
            bool requiresRelease, bool previousFire, bool previousReload, GunUseMode previousMode)
        {
            Profile = profile; Tick = tick; ShotSequence = sequence; Magazine = magazine; Reserve = reserve;
            FireRemaining = fire; EquipRemaining = equip; ReloadRemaining = reload; RecoveryRemaining = recovery;
            AimProgress = aim; RecoilPitch = pitch; RecoilYaw = yaw; RequiresRelease = requiresRelease;
            PreviousFire = previousFire; PreviousReload = previousReload; PreviousMode = previousMode;
        }
    }

    public static class Gunplay
    {
        public const int TickMilliseconds = 10;
        public const int MaximumReserve = 100000;
        public const long MaximumTick = long.MaxValue - 10000;

        public static bool TryCreate(GunProfile profile, long hostTick, int magazine, int reserve,
            out GunState state, out string error)
        {
            state = null;
            error = "Invalid gunplay initialization.";
            if (!ValidAmmo(profile, magazine, reserve) || hostTick < 0 || hostTick > MaximumTick) return false;
            state = new GunState(profile, hostTick, 0, magazine, reserve, 0, profile.EquipTicks, 0, 0,
                0, 0, 0, true, true, false, GunUseMode.Walking);
            error = null;
            return true;
        }

        public static bool TryBeginEquip(GunState state, out GunState next, out string error)
        {
            next = state;
            error = "Invalid gunplay state.";
            if (state == null) return false;
            next = new GunState(state.Profile, state.Tick, state.ShotSequence, state.Magazine, state.Reserve,
                state.FireRemaining, state.Profile.EquipTicks, 0, state.RecoveryRemaining, 0,
                state.RecoilPitch, state.RecoilYaw, true, true, state.PreviousReload, state.PreviousMode);
            error = null;
            return true;
        }

        public static bool TryReconcileAmmo(GunState state, int magazine, int reserve,
            out GunState next, out string error)
        {
            next = state;
            error = "Invalid ammunition snapshot.";
            if (state == null || !ValidAmmo(state.Profile, magazine, reserve)) return false;
            if (state.Magazine != magazine || state.Reserve != reserve)
                next = new GunState(state.Profile, state.Tick, state.ShotSequence, magazine, reserve,
                    state.FireRemaining, state.EquipRemaining, 0, state.RecoveryRemaining, state.AimProgress,
                    state.RecoilPitch, state.RecoilYaw, state.RequiresRelease, state.PreviousFire,
                    state.PreviousReload, state.PreviousMode);
            error = null;
            return true;
        }

        public static bool TryTick(GunState state, long hostTick, GunInput input, GunPose pose,
            out GunState next, out GunEvents events, out ShotIntent shot, out GunHandling handling, out string error)
        {
            next = state; events = GunEvents.None; shot = default(ShotIntent); handling = default(GunHandling);
            error = "Invalid gunplay tick.";
            if (state == null || hostTick > MaximumTick || hostTick != state.Tick + 1 ||
                pose.Mode < GunUseMode.Walking || pose.Mode > GunUseMode.Unavailable) return false;
            var p = state.Profile;
            int magazine = state.Magazine, reserve = state.Reserve;
            int fire = Down(state.FireRemaining), equip = Down(state.EquipRemaining);
            int recovery = Down(state.RecoveryRemaining), reload = Down(state.ReloadRemaining);
            int aim = state.AimProgress;
            double pitch = TowardZero(state.RecoilPitch, p.ReturnPerTick);
            double yaw = TowardZero(state.RecoilYaw, p.ReturnPerTick);
            bool release = state.RequiresRelease;
            long sequence = state.ShotSequence;
            bool walking = pose.Mode == GunUseMode.Walking;
            bool reloadEdge = input.Reload && !state.PreviousReload;
            bool fireEdge = input.Fire && !state.PreviousFire;

            if (!walking)
            {
                if (state.ReloadRemaining > 0) events |= GunEvents.ReloadCancelled;
                reload = 0; aim = 0; release = true;
            }
            else
            {
                if (state.PreviousMode != GunUseMode.Walking) recovery = p.RecoveryTicks;
                if (state.ReloadRemaining > 0 && reload == 0)
                {
                    int moved = Math.Min(p.MagazineCapacity - magazine, reserve);
                    magazine += moved; reserve -= moved;
                    events |= GunEvents.ReloadCompleted;
                }
                if (equip == 0 && recovery == 0 && reload == 0 && reloadEdge &&
                    magazine < p.MagazineCapacity && reserve > 0)
                {
                    reload = p.ReloadTicks;
                    events |= GunEvents.ReloadStarted;
                }
            }

            bool ready = walking && equip == 0 && reload == 0 && recovery == 0;
            if (walking && !input.Fire) release = false;
            if (ready && input.Aim) aim = Math.Min(p.AdsTicks, aim + 1);
            else aim = Math.Max(0, aim - 1);
            double aimFraction = (double)aim / p.AdsTicks;
            double spread = Spread(p, aimFraction, pose);
            if (ready && !release && fire == 0 && input.Fire &&
                (p.FireMode == GunFireMode.Automatic || fireEdge))
            {
                fire = p.FireTicks;
                if (magazine == 0) events |= GunEvents.DryFire;
                else
                {
                    magazine--; sequence++;
                    double yawKick = sequence % 2 == 1 ? p.YawKick : -p.YawKick;
                    double nextPitch = Math.Min(p.PitchLimit, pitch + p.PitchKick);
                    double nextYaw = Math.Max(-p.YawLimit, Math.Min(p.YawLimit, yaw + yawKick));
                    shot = new ShotIntent(sequence, hostTick, spread, nextPitch - pitch, nextYaw - yaw);
                    pitch = nextPitch; yaw = nextYaw;
                    events |= GunEvents.Fired;
                }
            }
            double movement = walking ? p.WalkScale + (p.AimScale - p.WalkScale) * aimFraction :
                (pose.Mode == GunUseMode.Sprinting ? p.SprintScale : 1);
            next = new GunState(p, hostTick, sequence, magazine, reserve, fire, equip, reload, recovery,
                aim, pitch, yaw, release, input.Fire, input.Reload, pose.Mode);
            handling = new GunHandling(aimFraction, pitch, yaw, spread, movement);
            error = null;
            return true;
        }

        private static bool ValidAmmo(GunProfile p, int magazine, int reserve)
        {
            return p != null && magazine >= 0 && magazine <= p.MagazineCapacity && reserve >= 0 && reserve <= MaximumReserve;
        }
        private static int Down(int value) { return Math.Max(0, value - 1); }
        private static double TowardZero(double value, double amount)
        {
            return value >= 0 ? Math.Max(0, value - amount) : Math.Min(0, value + amount);
        }
        private static double Spread(GunProfile p, double aim, GunPose pose)
        {
            return Math.Min(p.SpreadLimit, p.HipSpread + (p.AimSpread - p.HipSpread) * aim +
                (pose.Moving ? p.MovingSpread : 0) + (pose.Airborne ? p.AirborneSpread : 0));
        }
    }
}
