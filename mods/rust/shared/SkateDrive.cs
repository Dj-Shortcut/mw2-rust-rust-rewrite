using System;

namespace Shortcut.RustMod
{
    public struct SkateDriveInput
    {
        public readonly bool Push;
        public readonly bool Brake;
        public readonly bool Jump;
        public readonly double Steer;
        public readonly double LookYaw;
        public SkateDriveInput(bool push, bool brake, bool jump, double steer, double lookYaw)
        { Push = push; Brake = brake; Jump = jump; Steer = steer; LookYaw = lookYaw; }
    }

    public struct SkateDriveState
    {
        public readonly double Yaw;
        public readonly double Speed;
        public readonly bool PreviousJump;
        public readonly bool Grounded;
        public readonly bool HasStep;
        public readonly bool TakeoffPending;
        public readonly SkateVector DesiredVelocity;
        public SkateDriveState(double yaw)
            : this(yaw, 0, false, false, false, false, default(SkateVector)) { }
        internal SkateDriveState(double yaw, double speed, bool previousJump, bool grounded,
                                 bool hasStep, bool takeoffPending, SkateVector desiredVelocity)
        { Yaw = yaw; Speed = speed; PreviousJump = previousJump; Grounded = grounded;
          HasStep = hasStep; TakeoffPending = takeoffPending; DesiredVelocity = desiredVelocity; }
    }

    public struct SkateDriveResult
    {
        public readonly SkateDriveState State;
        public readonly SkateVector DesiredVelocity;
        public readonly double BoardYaw;
        public readonly double BoardPitch;
        public readonly double BoardRoll;
        public readonly SkateEvents Events;
        internal SkateDriveResult(SkateDriveState state, SkateVector velocity, double yaw,
                                  double pitch, double roll, SkateEvents events)
        { State = state; DesiredVelocity = velocity; BoardYaw = yaw;
          BoardPitch = pitch; BoardRoll = roll; Events = events; }
    }

    public static class SkateDrive
    {
        public const double LookTurnRate = 90;
        public const double SteerTurnRate = 90;
        public const double SteeringSpeed = 4;
        private static readonly SkateVector Up = new SkateVector(0, 1, 0);

        public static bool TryStep(SkateDriveState state, SkateDriveInput input, SkateVector actualVelocity,
                                   bool grounded, SkateVector groundNormal, double dt, double maxSpeed,
                                   out SkateDriveResult result, out string error)
        {
            result = new SkateDriveResult(state, state.DesiredVelocity, state.Yaw, 0, 0, SkateEvents.None);
            error = null;
            if (!Valid(state) || !Axis(input.Steer) || !Finite(input.LookYaw) || Math.Abs(input.LookYaw) > 360000 ||
                !Vector(actualVelocity) || !Vector(groundNormal) || !Finite(dt) || dt <= 0 || dt > SkateMotion.MaximumStep ||
                !Finite(maxSpeed) || maxSpeed <= 0 || maxSpeed > SkateMotion.MaximumGroundSpeed ||
                (grounded && (Math.Abs(groundNormal.LengthSquared - 1) > 0.0001 || groundNormal.Y < SkateMotion.WalkableNormalY)))
            { error = "Invalid skate drive step."; return false; }

            var actualHorizontal = new SkateVector(actualVelocity.X, 0, actualVelocity.Z);
            double observedSpeed = actualHorizontal.Length;
            double turnScale = Math.Min(1, observedSpeed / SteeringSpeed);
            double lookTurn = Clamp(Angle(input.LookYaw - state.Yaw), LookTurnRate * turnScale * dt);
            double yaw = Yaw(state.Yaw + lookTurn + input.Steer * SteerTurnRate * turnScale * dt);
            var events = SkateEvents.None;
            if (state.HasStep && !state.Grounded && grounded) events |= SkateEvents.Landed;
            if (state.HasStep && state.Speed > 0.1)
            {
                var requestedHorizontal = new SkateVector(state.DesiredVelocity.X, 0, state.DesiredVelocity.Z);
                double achieved = SkateVector.Dot(actualHorizontal, requestedHorizontal) / state.Speed;
                if (state.Speed - achieved > Math.Max(0.1, state.Speed * 0.25)) events |= SkateEvents.Blocked;
            }

            SkateVector velocity;
            double pitch = 0, roll = 0;
            bool takeoffPending = state.TakeoffPending && grounded && actualVelocity.Y > 0;
            if (grounded && !takeoffPending)
            {
                var normal = Unit(groundNormal);
                var heading = Forward(yaw);
                var tangent = Unit(new SkateVector(heading.X,
                    -(heading.X * normal.X + heading.Z * normal.Z) / normal.Y, heading.Z));
                double horizontalScale = Math.Sqrt(tangent.X * tangent.X + tangent.Z * tangent.Z);
                double speed = Math.Max(0, SkateVector.Dot(actualHorizontal, heading));
                speed = Math.Min(maxSpeed, speed);
                double acceleration = input.Push && !input.Brake ? SkateMotion.PushAcceleration : 0;
                acceleration += SkateVector.Dot(Up * -SkateMotion.Gravity, tangent);
                acceleration -= input.Brake ? SkateMotion.BrakeDeceleration : SkateMotion.RollingDeceleration;
                speed = Math.Min(maxSpeed, Math.Max(0, speed + acceleration * horizontalScale * dt));
                velocity = tangent * (speed / horizontalScale);
                var right = new SkateVector(heading.Z, 0, -heading.X);
                var pitchedUp = new SkateVector(-heading.X * tangent.Y, horizontalScale, -heading.Z * tangent.Y);
                pitch = Math.Atan2(-tangent.Y, horizontalScale) * 180 / Math.PI;
                roll = Math.Atan2(-SkateVector.Dot(normal, right), SkateVector.Dot(normal, pitchedUp)) * 180 / Math.PI;
                if (input.Jump && !state.PreviousJump)
                {
                    velocity = new SkateVector(velocity.X, SkateMotion.JumpSpeed - SkateMotion.Gravity * dt, velocity.Z);
                    pitch = 0;
                    roll = 0;
                    takeoffPending = true;
                    events |= SkateEvents.Jumped;
                }
            }
            else
            {
                if (observedSpeed > maxSpeed) actualHorizontal = actualHorizontal * (maxSpeed / observedSpeed);
                double vertical = Math.Max(-SkateMotion.MaximumFallSpeed, actualVelocity.Y - SkateMotion.Gravity * dt);
                velocity = new SkateVector(actualHorizontal.X, vertical, actualHorizontal.Z);
            }
            double horizontalSpeed = Math.Sqrt(velocity.X * velocity.X + velocity.Z * velocity.Z);
            var next = new SkateDriveState(yaw, Math.Min(maxSpeed, horizontalSpeed), input.Jump, grounded, true, takeoffPending, velocity);
            result = new SkateDriveResult(next, velocity, yaw, pitch, roll, events);
            return true;
        }

        private static bool Valid(SkateDriveState state)
        {
            return Finite(state.Yaw) && Math.Abs(state.Yaw) <= 360000 && Finite(state.Speed) && state.Speed >= 0 &&
                state.Speed <= SkateMotion.MaximumGroundSpeed && Vector(state.DesiredVelocity) &&
                Math.Abs(state.Speed * state.Speed - state.DesiredVelocity.X * state.DesiredVelocity.X -
                         state.DesiredVelocity.Z * state.DesiredVelocity.Z) < 0.000001;
        }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
        private static bool Vector(SkateVector value)
        { return Finite(value.X) && Finite(value.Y) && Finite(value.Z) && Math.Abs(value.X) <= 100000 && Math.Abs(value.Y) <= 100000 && Math.Abs(value.Z) <= 100000; }
        private static bool Axis(double value) { return Finite(value) && Math.Abs(value) <= 1; }
        private static double Clamp(double value, double limit) { return Math.Max(-limit, Math.Min(limit, value)); }
        private static double Yaw(double value) { var result = value % 360; return result < 0 ? result + 360 : result; }
        private static double Angle(double value) { var result = Yaw(value); return result > 180 ? result - 360 : result; }
        private static SkateVector Forward(double yaw) { var radians = yaw * Math.PI / 180; return new SkateVector(Math.Sin(radians), 0, Math.Cos(radians)); }
        private static SkateVector Unit(SkateVector value) { var length = value.Length; return length > 1e-12 ? value * (1 / length) : default(SkateVector); }
    }
}
