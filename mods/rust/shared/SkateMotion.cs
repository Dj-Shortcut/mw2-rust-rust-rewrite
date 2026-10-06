using System;

namespace Shortcut.RustMod
{
    public struct SkateVector
    {
        public readonly double X;
        public readonly double Y;
        public readonly double Z;

        public SkateVector(double x, double y, double z) { X = x; Y = y; Z = z; }
        public double LengthSquared { get { return Dot(this, this); } }
        public double Length { get { return Math.Sqrt(LengthSquared); } }
        public static SkateVector operator +(SkateVector a, SkateVector b) { return new SkateVector(a.X + b.X, a.Y + b.Y, a.Z + b.Z); }
        public static SkateVector operator -(SkateVector a, SkateVector b) { return new SkateVector(a.X - b.X, a.Y - b.Y, a.Z - b.Z); }
        public static SkateVector operator *(SkateVector a, double b) { return new SkateVector(a.X * b, a.Y * b, a.Z * b); }
        public static double Dot(SkateVector a, SkateVector b) { return a.X * b.X + a.Y * b.Y + a.Z * b.Z; }
    }

    public struct SkateHull
    {
        public readonly SkateVector HalfExtents;
        public readonly SkateVector CentreOffset;
        internal SkateHull(SkateVector extents, SkateVector offset) { HalfExtents = extents; CentreOffset = offset; }
    }

    public struct SkateHit
    {
        public readonly bool HasHit;
        public readonly bool StartedSolid;
        public readonly double Fraction;
        public readonly SkateVector Normal;
        public SkateHit(bool hasHit, bool startedSolid, double fraction, SkateVector normal)
        { HasHit = hasHit; StartedSolid = startedSolid; Fraction = fraction; Normal = normal; }
        public static SkateHit Miss { get { return new SkateHit(false, false, 1, default(SkateVector)); } }
    }

    public interface ISkateWorld
    {
        bool TrySweep(SkateVector start, SkateVector end, SkateHull hull, out SkateHit hit);
        bool TryClear(SkateVector position, SkateHull hull, out bool clear);
    }

    public struct SkateInput
    {
        public readonly bool Push;
        public readonly bool Brake;
        public readonly bool Jump;
        public readonly double Steer;
        public readonly double Spin;
        public readonly double Flip;
        public SkateInput(bool push, bool brake, bool jump, double steer, double spin, double flip)
        { Push = push; Brake = brake; Jump = jump; Steer = steer; Spin = spin; Flip = flip; }
    }

    public enum SkateMode { Grounded, Airborne, Bailed }
    [Flags]
    public enum SkateEvents { None = 0, Jumped = 1, Landed = 2, Bailed = 4, Blocked = 8 }

    public sealed class SkateState
    {
        public SkateVector Position { get; private set; }
        public SkateVector Velocity { get; private set; }
        public SkateVector GroundNormal { get; private set; }
        public double Yaw { get; private set; }
        public double Spin { get; private set; }
        public double Flip { get; private set; }
        public int TravelSign { get; private set; }
        public bool PreviousJump { get; private set; }
        public SkateMode Mode { get; private set; }

        internal SkateState(SkateVector position, SkateVector velocity, SkateVector normal, double yaw,
                            double spin, double flip, int travelSign, bool previousJump, SkateMode mode)
        { Position = position; Velocity = velocity; GroundNormal = normal; Yaw = yaw;
          Spin = spin; Flip = flip; TravelSign = travelSign; PreviousJump = previousJump; Mode = mode; }
    }

    public static class SkateMotion
    {
        public const double MaximumStep = 0.05;
        public const double MaximumGroundSpeed = 14;
        public const double PushAcceleration = 7;
        public const double BrakeDeceleration = 18;
        public const double RollingDeceleration = 0.4;
        public const double JumpSpeed = 5.2;
        public const double Gravity = 18;
        public const double MaximumFallSpeed = 40;
        public const double MaximumLandingFallSpeed = 8;
        public const double RotationRate = 360;
        public const double FlipRate = 720;
        public const double Skin = 0.002;
        public const double WalkableNormalY = 0.7071067811865476;
        public static readonly SkateHull MountedHull = new SkateHull(new SkateVector(0.55, 0.95, 0.55), new SkateVector(0, 0.8, 0));
        public static readonly SkateHull StandingHull = new SkateHull(new SkateVector(0.4, 0.9, 0.4), new SkateVector(0, 0.75, 0));
        private static readonly SkateVector Up = new SkateVector(0, 1, 0);

        public static bool TryMount(SkateVector position, double yaw, ISkateWorld world, out SkateState state, out string error)
        {
            state = null;
            error = null;
            if (!Coordinate(position) || !Finite(yaw) || Math.Abs(yaw) > 360000 || world == null)
                return Fail("Invalid mount request.", out error);
            bool clear;
            if (!Clear(world, position, MountedHull, out clear, out error)) return false;
            if (!clear) return Fail("The rider space is blocked.", out error);
            SkateVector supported, normal;
            bool found;
            if (!Support(world, position, MountedHull, 0.3, out found, out supported, out normal, out error)) return false;
            if (!found) return Fail("No nearby walkable support.", out error);
            if (!Clear(world, supported, MountedHull, out clear, out error)) return false;
            if (!clear) return Fail("The supported rider space is blocked.", out error);
            state = new SkateState(supported, default(SkateVector), normal, Yaw(yaw), 0, 0, 1, false, SkateMode.Grounded);
            return true;
        }

        public static bool TryStep(SkateState state, SkateInput input, double dt, ISkateWorld world,
                                   out SkateState next, out SkateEvents events, out string error)
        {
            next = state;
            events = SkateEvents.None;
            error = null;
            if (!Valid(state) || !Axis(input.Steer) || !Axis(input.Spin) || !Axis(input.Flip) ||
                !Finite(dt) || dt <= 0 || dt > MaximumStep || world == null)
                return Fail("Invalid skate step.", out error);
            if (state.Mode == SkateMode.Bailed) return Fail("A bailed rider must dismount or remount.", out error);
            bool clear;
            if (!Clear(world, state.Position, MountedHull, out clear, out error)) return false;
            if (!clear) return Fail("The rider space is blocked.", out error);
            var position = state.Position;
            var velocity = state.Velocity;
            var normal = state.GroundNormal;
            var mode = state.Mode;
            var yaw = state.Yaw;
            var spin = state.Spin;
            var flip = state.Flip;
            var sign = state.TravelSign;
            var pending = SkateEvents.None;

            if (mode == SkateMode.Grounded)
            {
                bool found;
                if (!Support(world, position, MountedHull, 0.12, out found, out position, out normal, out error)) return false;
                if (!found) mode = SkateMode.Airborne;
                else
                {
                    double speed = Math.Min(MaximumGroundSpeed, velocity.Length);
                    if (input.Push && !input.Brake) speed += PushAcceleration * dt;
                    var downhill = Project(Forward(yaw) * sign, normal);
                    speed += SkateVector.Dot(Up * -Gravity, Unit(downhill)) * dt;
                    bool reverse = speed < 0;
                    speed = Math.Abs(speed);
                    double deceleration = input.Brake ? BrakeDeceleration : RollingDeceleration;
                    speed = Math.Max(0, speed - deceleration * dt);
                    speed = Math.Min(MaximumGroundSpeed, speed);
                    if (reverse && speed > 0) sign = -sign;
                    yaw = Yaw(yaw + input.Steer * 90 * Math.Min(1, speed / 4) * dt * sign);
                    var tangent = Project(Forward(yaw) * sign, normal);
                    velocity = Unit(tangent) * speed;
                    if (input.Jump && !state.PreviousJump)
                    {
                        velocity = new SkateVector(velocity.X, JumpSpeed, velocity.Z);
                        mode = SkateMode.Airborne;
                        pending |= SkateEvents.Jumped;
                    }
                }
            }
            if (mode == SkateMode.Airborne)
            {
                velocity = new SkateVector(velocity.X, Math.Max(-MaximumFallSpeed, velocity.Y - Gravity * dt), velocity.Z);
                spin = Angle(spin + input.Spin * RotationRate * dt);
                flip = Angle(flip + input.Flip * FlipRate * dt);
            }

            var remaining = velocity * dt;
            for (int iteration = 0; iteration < 3 && remaining.LengthSquared > 1e-12; iteration++)
            {
                SkateHit hit;
                var end = position + remaining;
                if (!Sweep(world, position, end, MountedHull, out hit, out error)) return false;
                if (!hit.HasHit) { position = end; remaining = default(SkateVector); break; }
                position = position + remaining * hit.Fraction + hit.Normal * Skin;
                if (hit.Normal.Y >= WalkableNormalY && SkateVector.Dot(remaining, hit.Normal) < 0)
                {
                    if (mode == SkateMode.Airborne)
                    {
                        if (-SkateVector.Dot(velocity, hit.Normal) > MaximumLandingFallSpeed || Math.Abs(flip) > 25)
                        { mode = SkateMode.Bailed; pending |= SkateEvents.Bailed; break; }
                        var flat = new SkateVector(velocity.X, 0, velocity.Z);
                        var facing = Forward(Yaw(yaw + spin));
                        double alignment = flat.Length < 0.1 ? 1 : SkateVector.Dot(Unit(flat), facing);
                        if (Math.Abs(alignment) < WalkableNormalY)
                        { mode = SkateMode.Bailed; pending |= SkateEvents.Bailed; break; }
                        yaw = Yaw(yaw + spin);
                        sign = alignment < 0 ? -1 : 1;
                        spin = 0;
                        flip = 0;
                        mode = SkateMode.Grounded;
                        pending |= SkateEvents.Landed;
                    }
                    normal = hit.Normal;
                    velocity = Project(velocity, normal);
                    if (velocity.Length > MaximumGroundSpeed) velocity = Unit(velocity) * MaximumGroundSpeed;
                    remaining = Project(remaining * (1 - hit.Fraction), normal);
                    if (hit.Fraction <= 1e-8) { remaining = default(SkateVector); break; }
                }
                else
                {
                    pending |= SkateEvents.Blocked;
                    if (mode == SkateMode.Airborne || velocity.Length > 4)
                    { mode = SkateMode.Bailed; pending |= SkateEvents.Bailed; }
                    velocity = default(SkateVector);
                    remaining = default(SkateVector);
                }
            }
            if (mode == SkateMode.Grounded)
            {
                bool found;
                if (!Support(world, position, MountedHull, 0.12, out found, out position, out normal, out error)) return false;
                if (!found) mode = SkateMode.Airborne;
                else velocity = Project(velocity, normal);
            }
            if (mode == SkateMode.Bailed)
            { velocity = default(SkateVector); spin = 0; flip = 0; }
            var candidate = new SkateState(position, velocity, normal, yaw, spin, flip, sign, input.Jump, mode);
            if (!Valid(candidate)) return Fail("Skate movement exceeds the allowed bounds.", out error);
            if (!Clear(world, position, MountedHull, out clear, out error)) return false;
            if (!clear) return Fail("The final rider space is blocked.", out error);
            next = candidate;
            events = pending;
            return true;
        }

        public static bool TryDismount(SkateState state, ISkateWorld world, out SkateVector feet, out string error)
        {
            feet = default(SkateVector);
            error = null;
            if (!Valid(state) || world == null) return Fail("Invalid dismount request.", out error);
            var right = Forward(state.Yaw + 90);
            for (int side = 1; side >= -1; side -= 2)
            {
                var candidate = state.Position + right * (side * 1.2);
                SkateHit hit;
                if (!Sweep(world, state.Position, candidate, StandingHull, out hit, out error)) return false;
                if (hit.HasHit) continue;
                bool found, clear;
                SkateVector supported, normal;
                if (!Support(world, candidate, StandingHull, 0.3, out found, out supported, out normal, out error)) return false;
                if (!found) continue;
                if (!Clear(world, supported, StandingHull, out clear, out error)) return false;
                if (!clear) continue;
                var standingFeet = supported - Up * 0.15;
                if (!Coordinate(standingFeet)) return Fail("Dismount exceeds the allowed bounds.", out error);
                feet = standingFeet;
                return true;
            }
            return Fail("No clear supported dismount space.", out error);
        }

        private static bool Support(ISkateWorld world, SkateVector position, SkateHull hull, double reach,
                                    out bool found, out SkateVector supported, out SkateVector normal, out string error)
        {
            found = false;
            supported = position;
            normal = Up;
            SkateHit hit;
            var start = position;
            var end = position - Up * reach;
            if (!Sweep(world, start, end, hull, out hit, out error)) return false;
            if (!hit.HasHit || hit.Normal.Y < WalkableNormalY) return true;
            supported = start + (end - start) * hit.Fraction + Up * (Skin / hit.Normal.Y);
            normal = hit.Normal;
            found = true;
            return true;
        }

        private static bool Sweep(ISkateWorld world, SkateVector start, SkateVector end, SkateHull hull,
                                  out SkateHit hit, out string error)
        {
            hit = SkateHit.Miss;
            error = null;
            if (!Coordinate(start) || !Coordinate(end)) return Fail("World query exceeds the allowed bounds.", out error);
            try
            {
                if (!world.TrySweep(start, end, hull, out hit)) return Fail("World collision query failed.", out error);
            }
            catch (Exception) { return Fail("World collision query failed.", out error); }
            if (hit.StartedSolid) return Fail("World collision query starts inside solid space.", out error);
            if (!Finite(hit.Fraction) || hit.Fraction < 0 || hit.Fraction > 1)
                return Fail("Invalid world collision fraction.", out error);
            if (!hit.HasHit)
            {
                if (hit.Fraction != 1) return Fail("Invalid empty world collision query.", out error);
                return true;
            }
            if (!Vector(hit.Normal) || Math.Abs(hit.Normal.LengthSquared - 1) > 0.0001 ||
                SkateVector.Dot(end - start, hit.Normal) > 0.000001)
                return Fail("Invalid world collision normal.", out error);
            return true;
        }

        private static bool Clear(ISkateWorld world, SkateVector position, SkateHull hull, out bool clear, out string error)
        {
            clear = false;
            error = null;
            if (!Coordinate(position)) return Fail("World query exceeds the allowed bounds.", out error);
            try
            {
                if (!world.TryClear(position, hull, out clear)) return Fail("World clearance query failed.", out error);
            }
            catch (Exception) { return Fail("World clearance query failed.", out error); }
            return true;
        }

        private static bool Valid(SkateState state)
        {
            return state != null && Coordinate(state.Position) && Vector(state.Velocity) &&
                Math.Abs(state.Velocity.Y) <= MaximumFallSpeed &&
                state.Velocity.X * state.Velocity.X + state.Velocity.Z * state.Velocity.Z <= MaximumGroundSpeed * MaximumGroundSpeed + 0.000001 &&
                Vector(state.GroundNormal) && Math.Abs(state.GroundNormal.LengthSquared - 1) < 0.0001 &&
                Finite(state.Yaw) && state.Yaw >= 0 && state.Yaw < 360 && Axis(state.Spin / 180) && Axis(state.Flip / 180) &&
                (state.TravelSign == -1 || state.TravelSign == 1) &&
                (state.Mode == SkateMode.Grounded || state.Mode == SkateMode.Airborne || state.Mode == SkateMode.Bailed);
        }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
        private static bool Vector(SkateVector value) { return Finite(value.X) && Finite(value.Y) && Finite(value.Z); }
        private static bool Coordinate(SkateVector value) { return Vector(value) && Math.Abs(value.X) <= 100000 && Math.Abs(value.Y) <= 100000 && Math.Abs(value.Z) <= 100000; }
        private static bool Axis(double value) { return Finite(value) && Math.Abs(value) <= 1; }
        private static double Yaw(double value) { var result = value % 360; return result < 0 ? result + 360 : result; }
        private static double Angle(double value) { var result = Yaw(value); return result > 180 ? result - 360 : result; }
        private static SkateVector Forward(double yaw) { var radians = yaw * Math.PI / 180; return new SkateVector(Math.Sin(radians), 0, Math.Cos(radians)); }
        private static SkateVector Unit(SkateVector value) { var length = value.Length; return length > 1e-12 ? value * (1 / length) : default(SkateVector); }
        private static SkateVector Project(SkateVector value, SkateVector normal) { return value - normal * SkateVector.Dot(value, normal); }
        private static bool Fail(string message, out string error) { error = message; return false; }
    }
}
