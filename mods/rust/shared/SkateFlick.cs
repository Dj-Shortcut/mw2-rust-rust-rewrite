using System;

namespace Shortcut.RustMod
{
    [Flags]
    public enum SkateFlickEvents
    {
        None = 0,
        Ollie = 1,
        Nollie = 2,
        Kickflip = 4,
        Heelflip = 8,
        ShoveFrontside = 16,
        ShoveBackside = 32
    }

    public struct SkateFlickInput
    {
        public readonly double X;
        public readonly double Y;
        public readonly bool Airborne;
        public readonly bool Mirrored;

        public SkateFlickInput(double x, double y, bool airborne, bool mirrored)
        { X = x; Y = y; Airborne = airborne; Mirrored = mirrored; }
    }

    public struct SkateFlickState
    {
        internal int Phase;
        internal bool Forward;
        internal double SetSeconds;
        internal double HeldSeconds;
        internal double FlickSeconds;
        internal double RimSeconds;
        internal double BestX;
        internal double BestY;
        internal double BandSeconds;
        internal double OutSeconds;
        internal bool Manual;
        internal bool NoseManual;
    }

    public struct SkateFlickResult
    {
        public readonly SkateFlickState State;
        public readonly SkateFlickEvents Events;
        public readonly double Pop;
        public readonly double Charge;
        public readonly bool Charging;
        public readonly bool Manual;
        public readonly bool NoseManual;

        public SkateFlickResult(SkateFlickState state, SkateFlickEvents events, double pop, double charge, bool charging, bool manual, bool noseManual)
        { State = state; Events = events; Pop = pop; Charge = charge; Charging = charging; Manual = manual; NoseManual = noseManual; }
    }

    // The right stick as in the skate. games: held back and flicked forward is an ollie, forward and
    // back a nollie; where the flick arrives picks the trick. Each step says where the stick has
    // been for the last dt seconds, so a caller that only hears of changes steps once per change
    // with the position that just ended.
    public static class SkateFlick
    {
        public const double MaximumStep = 0.1;
        public const double DeadZone = 0.25;
        public const double HeldZone = 0.7;
        public const double HeldSpreadDegrees = 40;
        public const double SetSeconds = 0.03;
        public const double FlickSeconds = 0.3;
        public const double FlickReach = 0.45;
        public const double FireReach = 0.8;
        public const double FlickSide = 0.15;
        public const double ShoveSide = 0.2;
        public const double SettleSeconds = 0.025;
        public const double PeakDrop = 0.15;
        public const double StraightSpreadDegrees = 22.5;
        public const double FlipSpreadDegrees = 67.5;
        public const double AirFlickSeconds = 0.18;
        public const double AirSpreadDegrees = 112.5;
        public const double ManualFrom = 0.25;
        public const double ManualTo = 0.62;
        public const double ManualSide = 0.35;
        public const double ManualSeconds = 0.2;
        public const double ManualGrace = 0.06;
        public const double FullCharge = 0.3;
        public const double MinimumPop = 0.6;

        private const int Rest = 0, Held = 1, Flicking = 2, Spent = 3, AirFlick = 4;

        public static bool TryStep(SkateFlickState state, SkateFlickInput input, double dt, out SkateFlickResult result, out string error)
        {
            result = default(SkateFlickResult);
            error = null;
            if (!Finite(input.X) || !Finite(input.Y) || Math.Abs(input.X) > 1.0001 || Math.Abs(input.Y) > 1.0001 ||
                !(dt > 0) || dt > MaximumStep || state.Phase < Rest || state.Phase > AirFlick)
            { error = "Invalid skate flick request."; return false; }

            double x = input.X, y = input.Y;
            double far = Math.Sqrt(x * x + y * y);
            bool heldBack = far >= HeldZone && Degrees(Math.Abs(x), -y) <= HeldSpreadDegrees;
            bool heldForward = far >= HeldZone && Degrees(Math.Abs(x), y) <= HeldSpreadDegrees;
            var events = SkateFlickEvents.None;
            double pop = 0;

            if (state.Phase == Rest)
            {
                if (heldBack || heldForward)
                {
                    if (state.SetSeconds > 0 && state.Forward != heldForward) state.SetSeconds = 0;
                    state.Forward = heldForward;
                    state.SetSeconds += dt;
                    if (state.SetSeconds >= SetSeconds) { state.Phase = Held; state.HeldSeconds = state.SetSeconds; }
                }
                else
                {
                    state.SetSeconds = 0;
                    // In the air a flick ahead or to a side adds a flip. A stick on its way back is
                    // being set for the next pop, and is left to get there.
                    if (input.Airborne && far > DeadZone && Degrees(Math.Abs(x), y) <= AirSpreadDegrees) { state.Phase = AirFlick; state.FlickSeconds = 0; }
                }
            }
            else if (state.Phase == Held)
            {
                if (state.Forward ? heldForward : heldBack) state.HeldSeconds += dt;
                else { state.Phase = Flicking; state.FlickSeconds = 0; state.RimSeconds = 0; state.BestX = 0; state.BestY = 0; }
            }

            if (state.Phase == Flicking)
            {
                // Seen from the side the flick goes to: toward is positive once the stick has crossed.
                double toward = state.Forward ? -y : y;
                if (state.Forward ? heldForward : heldBack) state.Phase = Held;
                else
                {
                    state.FlickSeconds += dt;
                    double best = Math.Sqrt(state.BestX * state.BestX + state.BestY * state.BestY);
                    if (toward > FlickSide && far > best) { state.BestX = x; state.BestY = y; best = far; }
                    // A stick that has come to the rim is read where it settles there: on its way
                    // round to a side it passes places that would read as another trick. One that
                    // turns back before the rim is read where it was furthest.
                    if (far >= FireReach && toward > -ShoveSide) state.RimSeconds += dt; else state.RimSeconds = 0;
                    bool settled = state.RimSeconds >= SettleSeconds;
                    bool turned = best >= FlickReach && (far < best - PeakDrop || state.FlickSeconds > FlickSeconds);
                    if (settled || turned)
                    {
                        events = (settled ? Pick(x, toward, input.Mirrored, true) : Pick(state.BestX, state.Forward ? -state.BestY : state.BestY, input.Mirrored, true)) |
                                 (state.Forward ? SkateFlickEvents.Nollie : SkateFlickEvents.Ollie);
                        pop = MinimumPop + (1 - MinimumPop) * Math.Min(1, state.HeldSeconds / FullCharge);
                        state.Phase = Spent;
                    }
                    else if (state.FlickSeconds > FlickSeconds) { state.Phase = Rest; state.SetSeconds = 0; }
                }
            }
            else if (state.Phase == AirFlick)
            {
                state.FlickSeconds += dt;
                if (!input.Airborne || far <= DeadZone) state.Phase = Rest;
                else if (state.FlickSeconds > AirFlickSeconds) state.Phase = Degrees(Math.Abs(x), y) <= AirSpreadDegrees ? Spent : Rest;
                else if (far >= FireReach)
                {
                    // Straight ahead is no flip: the stick may be on its way to be held there.
                    events = Pick(x, y, input.Mirrored, false);
                    state.Phase = events == SkateFlickEvents.None ? Rest : Spent;
                }
                if (state.Phase == Rest) state.SetSeconds = 0;
            }
            else if (state.Phase == Spent && far <= DeadZone) { state.Phase = Rest; state.SetSeconds = 0; }

            // Part of the way back, or forward, and kept there: the board is held on two wheels.
            bool charging = state.Phase == Held;
            bool backBand = -y >= ManualFrom && -y <= ManualTo && Math.Abs(x) <= ManualSide;
            bool forwardBand = y >= ManualFrom && y <= ManualTo && Math.Abs(x) <= ManualSide;
            bool keeps = (state.Manual && (backBand || (charging && !state.Forward))) || (state.NoseManual && (forwardBand || (charging && state.Forward)));
            if ((backBand || forwardBand) && state.Phase != Held && state.Phase != Flicking)
            {
                state.BandSeconds += dt; state.OutSeconds = 0;
                if (state.BandSeconds >= ManualSeconds) { state.Manual = backBand; state.NoseManual = forwardBand; }
            }
            else if (keeps) state.OutSeconds = 0;
            else
            {
                state.BandSeconds = 0;
                state.OutSeconds += dt;
                if (state.OutSeconds > ManualGrace || events != SkateFlickEvents.None) { state.Manual = false; state.NoseManual = false; }
            }

            double charge = charging ? Math.Min(1, state.HeldSeconds / FullCharge) : 0;
            result = new SkateFlickResult(state, events, pop, charge, charging, state.Manual, state.NoseManual);
            return true;
        }

        // x to the stick's right, toward the way the flick went. The heel side of a regular rider
        // is the stick's left: a kickflip leaves the board on that side.
        private static SkateFlickEvents Pick(double x, double toward, bool mirrored, bool popped)
        {
            double angle = Degrees(Math.Abs(x), toward);
            if (angle <= StraightSpreadDegrees) return SkateFlickEvents.None;
            bool heelSide = (x < 0) != mirrored;
            if (angle <= FlipSpreadDegrees || !popped)
                return angle > AirSpreadDegrees ? SkateFlickEvents.None : heelSide ? SkateFlickEvents.Kickflip : SkateFlickEvents.Heelflip;
            return heelSide ? SkateFlickEvents.ShoveBackside : SkateFlickEvents.ShoveFrontside;
        }

        private static double Degrees(double across, double along) { return Math.Atan2(across, along) * 180 / Math.PI; }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
    }
}
