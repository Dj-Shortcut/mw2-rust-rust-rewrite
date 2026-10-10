using System;

namespace Shortcut.RustMod
{
    [Flags]
    public enum SkateTrickEvents { None = 0, Landed = 1, Bailed = 2, GrindEnded = 4, ComboBanked = 8, ManualEnded = 16 }

    public struct SkateTrickInput
    {
        public readonly double Spin, Flip, ImpactSpeed, RollingSpeed;
        public readonly bool Grab, Airborne, Grinding, Manual;
        public SkateTrickInput(double spin, double flip, bool grab, bool airborne, bool grinding, double impactSpeed)
            : this(spin, flip, grab, airborne, grinding, impactSpeed, false, 0) { }
        public SkateTrickInput(double spin, double flip, bool grab, bool airborne, bool grinding,
                               double impactSpeed, bool manual, double rollingSpeed)
        { Spin = spin; Flip = flip; Grab = grab; Airborne = airborne; Grinding = grinding;
          ImpactSpeed = impactSpeed; Manual = manual; RollingSpeed = rollingSpeed; }
    }

    public struct SkateTrickState
    {
        public readonly double Spin, Flip, GrindSeconds, RollingSeconds, ManualSeconds;
        public readonly bool Airborne, Grinding, Grabbed, Switch, Manualing;
        public readonly long ComboBasePoints, TotalPoints;
        public readonly int ComboCount;
        public SkateTrickState(bool ridingSwitch) : this(0, 0, 0, 0, false, false, false, ridingSwitch, 0, 0, 0) { }
        internal SkateTrickState(double spin, double flip, double grind, double rolling, bool air, bool grinding,
                                 bool grab, bool ridingSwitch, long combo, long total, int count)
            : this(spin, flip, grind, rolling, air, grinding, grab, ridingSwitch, combo, total, count, 0, false) { }
        internal SkateTrickState(double spin, double flip, double grind, double rolling, bool air, bool grinding,
                                 bool grab, bool ridingSwitch, long combo, long total, int count,
                                 double manualSeconds, bool manualing)
        { Spin = spin; Flip = flip; GrindSeconds = grind; RollingSeconds = rolling; Airborne = air;
          Grinding = grinding; Grabbed = grab; Switch = ridingSwitch; ComboBasePoints = combo;
          TotalPoints = total; ComboCount = count; ManualSeconds = manualSeconds; Manualing = manualing; }
    }

    public struct SkateTrickResult
    {
        public readonly SkateTrickState State;
        public readonly double BoardSpin, BoardFlip, GrindSeconds, ManualSeconds;
        public readonly SkateTrickEvents Events;
        public readonly string TrickName;
        public readonly long Points, ComboPoints, BankedPoints;
        public bool Switch { get { return State.Switch; } }
        public bool Manualing { get { return State.Manualing; } }
        public long TotalPoints { get { return State.TotalPoints; } }
        internal SkateTrickResult(SkateTrickState state, double spin, double flip, double grind, SkateTrickEvents events,
                                  string name, long points, long banked)
            : this(state, spin, flip, grind, state.ManualSeconds, events, name, points, banked) { }
        internal SkateTrickResult(SkateTrickState state, double spin, double flip, double grind, double manualSeconds,
                                  SkateTrickEvents events, string name, long points, long banked)
        { State = state; BoardSpin = spin; BoardFlip = flip; GrindSeconds = grind;
          ManualSeconds = manualSeconds; Events = events; TrickName = name; Points = points;
          ComboPoints = SkateTricks.LiveComboValue(state); BankedPoints = banked; }
    }

    public static class SkateTricks
    {
        public const double FlipTolerance = 25;
        public const double SpinTolerance = 25;
        public const double ComboRollingTimeout = 1.5;
        public const int MaximumMultiplier = 16;
        public const double MinimumManualSpeed = 0.01;
        public const int ManualPointsPerSecond = 100;
        private const double MaximumManualSeconds = 86400;
        private const long PointsLimit = 1000000000000000;
        private static readonly string[] Names = {
            "Ollie", "Kickflip", "Heelflip", "Double Kickflip", "Double Heelflip",
            "Frontside 180", "Frontside 180 Kickflip", "Frontside 180 Heelflip", "Frontside 180 Double Kickflip", "Frontside 180 Double Heelflip",
            "Backside 180", "Backside 180 Kickflip", "Backside 180 Heelflip", "Backside 180 Double Kickflip", "Backside 180 Double Heelflip",
            "Frontside 360", "Frontside 360 Kickflip", "Frontside 360 Heelflip", "Frontside 360 Double Kickflip", "Frontside 360 Double Heelflip",
            "Backside 360", "Backside 360 Kickflip", "Backside 360 Heelflip", "Backside 360 Double Kickflip", "Backside 360 Double Heelflip",
            "Ollie Grab", "Kickflip Grab", "Heelflip Grab", "Double Kickflip Grab", "Double Heelflip Grab",
            "Frontside 180 Grab", "Frontside 180 Kickflip Grab", "Frontside 180 Heelflip Grab", "Frontside 180 Double Kickflip Grab", "Frontside 180 Double Heelflip Grab",
            "Backside 180 Grab", "Backside 180 Kickflip Grab", "Backside 180 Heelflip Grab", "Backside 180 Double Kickflip Grab", "Backside 180 Double Heelflip Grab",
            "Frontside 360 Grab", "Frontside 360 Kickflip Grab", "Frontside 360 Heelflip Grab", "Frontside 360 Double Kickflip Grab", "Frontside 360 Double Heelflip Grab",
            "Backside 360 Grab", "Backside 360 Kickflip Grab", "Backside 360 Heelflip Grab", "Backside 360 Double Kickflip Grab", "Backside 360 Double Heelflip Grab"
        };

        public static bool TryStep(SkateTrickState state, SkateTrickInput input, double dt,
                                   out SkateTrickResult result, out string error)
        { return TryStep(state, input, dt, SkateMotion.MaximumLandingFallSpeed, out result, out error); }

        public static bool TryStep(SkateTrickState state, SkateTrickInput input, double dt, double maximumLandingImpact,
                                   out SkateTrickResult result, out string error)
        {
            result = new SkateTrickResult(state, state.Spin, state.Flip, state.GrindSeconds, SkateTrickEvents.None, null, 0, 0);
            error = null;
            if (!Valid(state) || !Axis(input.Spin) || !Axis(input.Flip) || !Finite(input.ImpactSpeed) || input.ImpactSpeed < 0 ||
                input.ImpactSpeed > 100000 || !Finite(input.RollingSpeed) || input.RollingSpeed < 0 || input.RollingSpeed > 100000 ||
                (input.Airborne && input.Grinding) || !Finite(dt) || dt <= 0 || dt > SkateMotion.MaximumStep ||
                !Finite(maximumLandingImpact) || maximumLandingImpact <= 0 || maximumLandingImpact > 100000)
            { error = "Invalid skate trick step."; return false; }
            double spin = state.Spin, flip = state.Flip, grind = state.GrindSeconds, rolling = state.RollingSeconds;
            double manual = state.ManualSeconds;
            long combo = state.ComboBasePoints, total = state.TotalPoints, points = 0, banked = 0;
            int count = state.ComboCount;
            bool grabbed = state.Grabbed, ridingSwitch = state.Switch;
            var events = SkateTrickEvents.None;
            string name = null;
            if (input.Airborne)
            {
                if (!state.Airborne) { spin = 0; flip = 0; grabbed = false; }
                spin = Math.Max(-3600000, Math.Min(3600000, spin + input.Spin * SkateMotion.RotationRate * dt));
                flip = Math.Max(-3600000, Math.Min(3600000, flip + input.Flip * SkateMotion.FlipRate * dt));
                grabbed |= input.Grab;
            }
            double visualSpin = spin, visualFlip = flip, completedGrind = grind, completedManual = manual;
            if (state.Airborne && !input.Airborne)
            {
                int turns = (int)Math.Round(flip / 360, MidpointRounding.AwayFromZero);
                int halves = (int)Math.Round(spin / 180, MidpointRounding.AwayFromZero);
                if (Math.Abs(flip - turns * 360) > FlipTolerance + 1e-8 || Math.Abs(spin - halves * 180) > SpinTolerance + 1e-8 ||
                    input.ImpactSpeed > maximumLandingImpact)
                {
                    events |= SkateTrickEvents.Bailed;
                    name = "Bail";
                    combo = 0; count = 0; rolling = 0;
                }
                else
                {
                    events |= SkateTrickEvents.Landed;
                    ridingSwitch ^= (Math.Abs(halves) % 2) != 0;
                    int flipIndex = turns == 0 ? 0 : Math.Abs(turns) == 1 ? (turns > 0 ? 1 : 2) : (turns > 0 ? 3 : 4);
                    int spinIndex = halves == 0 ? 0 : Math.Abs(halves) % 2 != 0 ? (halves > 0 ? 1 : 2) : (halves > 0 ? 3 : 4);
                    name = Math.Abs(turns) > 2 || Math.Abs(halves) > 2
                        ? (grabbed ? "Multiple Rotation Trick Grab" : "Multiple Rotation Trick")
                        : Names[spinIndex * 5 + flipIndex + (grabbed ? 25 : 0)];
                    points = 50 + Math.Min(2, Math.Abs(turns)) * 100 + Math.Min(2, Math.Abs(halves)) * 100 + (grabbed ? 50 : 0);
                    combo = Add(combo, points); count = Math.Min(MaximumMultiplier, count + 1);
                }
                spin = 0; flip = 0; grabbed = false;
            }
            if (state.Grinding && !input.Grinding)
            {
                points = Math.Max(1, (long)Math.Round(grind * 100));
                name = "Grind";
                events |= SkateTrickEvents.GrindEnded;
                combo = Add(combo, points); count = Math.Min(MaximumMultiplier, count + 1);
                grind = 0;
            }
            bool grinding = input.Grinding && (events & SkateTrickEvents.Bailed) == 0;
            if (grinding)
            {
                grind = Math.Min(86400, (state.Grinding ? grind : 0) + dt);
                completedGrind = grind;
            }
            bool manualing = input.Manual && !input.Airborne && !input.Grinding &&
                input.RollingSpeed > MinimumManualSpeed && (events & SkateTrickEvents.Bailed) == 0;
            if (state.Manualing && !manualing)
            {
                completedManual = manual;
                if ((events & SkateTrickEvents.Bailed) == 0)
                {
                    points = DurationPoints(manual);
                    name = "Manual";
                    events |= SkateTrickEvents.ManualEnded;
                    if (points > 0)
                    { combo = Add(combo, points); count = Math.Min(MaximumMultiplier, count + 1); }
                }
                manual = 0;
            }
            if (manualing)
            {
                manual = Math.Min(MaximumManualSeconds, (state.Manualing ? manual : 0) + dt);
                completedManual = manual;
                if (events == SkateTrickEvents.None)
                { name = "Manual"; points = DurationPoints(manual); }
            }
            if (input.Airborne || grinding || manualing || state.Airborne || state.Grinding || state.Manualing) rolling = 0;
            else if (count > 0)
            {
                rolling = Math.Min(ComboRollingTimeout, rolling + dt);
                if (rolling >= ComboRollingTimeout - 1e-9)
                {
                    banked = ComboValue(combo, count); total = Add(total, banked);
                    events |= SkateTrickEvents.ComboBanked;
                    name = "Combo"; points = banked;
                    combo = 0; count = 0; rolling = 0;
                }
            }
            var next = new SkateTrickState(spin, flip, grind, rolling, input.Airborne, grinding,
                grabbed, ridingSwitch, combo, total, count, manual, manualing);
            result = new SkateTrickResult(next, visualSpin, visualFlip, completedGrind, completedManual, events, name, points, banked);
            return true;
        }

        public static bool TryBail(SkateTrickState state, out SkateTrickResult result, out string error)
        {
            result = new SkateTrickResult(state, state.Spin, state.Flip, state.GrindSeconds, SkateTrickEvents.None, null, 0, 0);
            error = null;
            if (!Valid(state)) { error = "Invalid skate trick state."; return false; }
            var next = new SkateTrickState(0, 0, 0, 0, false, false, false, state.Switch, 0, state.TotalPoints, 0);
            result = new SkateTrickResult(next, 0, 0, state.GrindSeconds, state.ManualSeconds, SkateTrickEvents.Bailed, "Bail", 0, 0);
            return true;
        }

        internal static long ComboValue(long points, int count) { return Math.Min(PointsLimit, points * (long)count); }
        internal static long LiveComboValue(SkateTrickState state)
        {
            if (state.ComboBasePoints < 0 || state.ComboBasePoints > PointsLimit ||
                state.ComboCount < 0 || state.ComboCount > MaximumMultiplier) return 0;
            long points = state.ComboBasePoints;
            int count = state.ComboCount;
            if (state.Manualing && Finite(state.ManualSeconds) && state.ManualSeconds > 0 && state.ManualSeconds <= MaximumManualSeconds)
            { points = Add(points, DurationPoints(state.ManualSeconds)); count = Math.Min(MaximumMultiplier, count + 1); }
            return ComboValue(points, count);
        }
        private static long DurationPoints(double seconds)
        {
            if (!Finite(seconds) || seconds <= 0) return 0;
            return Math.Max(1, (long)Math.Round(Math.Min(MaximumManualSeconds, seconds) * ManualPointsPerSecond));
        }
        private static long Add(long a, long b) { return Math.Min(PointsLimit, a + b); }
        private static bool Valid(SkateTrickState s)
        { return Finite(s.Spin) && Finite(s.Flip) && Math.Abs(s.Spin) <= 3600000 && Math.Abs(s.Flip) <= 3600000 &&
            Finite(s.GrindSeconds) && s.GrindSeconds >= 0 && s.GrindSeconds <= 86400 && Finite(s.RollingSeconds) &&
            s.RollingSeconds >= 0 && s.RollingSeconds <= ComboRollingTimeout && s.ComboCount >= 0 && s.ComboCount <= MaximumMultiplier &&
            s.ComboBasePoints >= 0 && s.ComboBasePoints <= PointsLimit && s.TotalPoints >= 0 && s.TotalPoints <= PointsLimit &&
            !(s.Airborne && s.Grinding) && Finite(s.ManualSeconds) && s.ManualSeconds >= 0 && s.ManualSeconds <= MaximumManualSeconds &&
            (s.Manualing ? s.ManualSeconds > 0 && !s.Airborne && !s.Grinding : s.ManualSeconds == 0) &&
            (s.ComboCount == 0) == (s.ComboBasePoints == 0); }
        private static bool Axis(double n) { return Finite(n) && Math.Abs(n) <= 1; }
        private static bool Finite(double n) { return !double.IsNaN(n) && !double.IsInfinity(n); }
    }
}
