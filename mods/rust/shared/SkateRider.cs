using System;

namespace Shortcut.RustMod
{
    public struct SkateRiderRig
    {
        public readonly double PelvisHeight, HipWidth, ThighLength, ShinLength, FootLength;
        public readonly double SpineLength, ShoulderWidth, UpperArmLength, ForearmLength, NeckToHeadLength;
        public SkateRiderRig(double pelvisHeight, double hipWidth, double thighLength, double shinLength,
                             double footLength, double spineLength, double shoulderWidth, double upperArmLength,
                             double forearmLength, double neckToHeadLength)
        { PelvisHeight = pelvisHeight; HipWidth = hipWidth; ThighLength = thighLength; ShinLength = shinLength;
          FootLength = footLength; SpineLength = spineLength; ShoulderWidth = shoulderWidth;
          UpperArmLength = upperArmLength; ForearmLength = forearmLength; NeckToHeadLength = neckToHeadLength; }
    }

    public struct SkateRiderInput
    {
        public readonly SkateVector BoardPosition, BoardForward, BoardUp;
        public readonly SkateStance Stance;
        public readonly bool Switch, Airborne, Pushing, Grab, Bailed;
        public readonly double Speed, LeanDegrees, Crouch, PushPhase, FlipDegrees, LookYawDegrees;
        public SkateRiderInput(SkateVector boardPosition, SkateVector boardForward, SkateVector boardUp,
                               SkateStance stance, bool ridingSwitch, double speed, double leanDegrees,
                               double crouch, bool airborne, bool pushing, double pushPhase,
                               double flipDegrees, bool grab, bool bailed, double lookYawDegrees)
        { BoardPosition = boardPosition; BoardForward = boardForward; BoardUp = boardUp; Stance = stance;
          Switch = ridingSwitch; Speed = speed; LeanDegrees = leanDegrees; Crouch = crouch; Airborne = airborne;
          Pushing = pushing; PushPhase = pushPhase; FlipDegrees = flipDegrees; Grab = grab;
          Bailed = bailed; LookYawDegrees = lookYawDegrees; }
    }

    public struct SkateRiderPose
    {
        public readonly SkateVector Pelvis, LeftHip, RightHip, LeftKnee, RightKnee;
        public readonly SkateVector LeftAnkle, RightAnkle, LeftToe, RightToe, Chest, Neck, Head;
        public readonly SkateVector LeftShoulder, RightShoulder, LeftElbow, RightElbow, LeftHand, RightHand;
        public readonly SkateVector PelvisForward, PelvisUp, ChestForward, ChestUp, HeadForward;
        internal SkateRiderPose(SkateVector pelvis, SkateVector leftHip, SkateVector rightHip,
                                SkateVector leftKnee, SkateVector rightKnee, SkateVector leftAnkle,
                                SkateVector rightAnkle, SkateVector leftToe, SkateVector rightToe,
                                SkateVector chest, SkateVector neck, SkateVector head, SkateVector leftShoulder,
                                SkateVector rightShoulder, SkateVector leftElbow, SkateVector rightElbow,
                                SkateVector leftHand, SkateVector rightHand, SkateVector pelvisForward,
                                SkateVector pelvisUp, SkateVector chestForward, SkateVector chestUp,
                                SkateVector headForward)
        { Pelvis = pelvis; LeftHip = leftHip; RightHip = rightHip; LeftKnee = leftKnee; RightKnee = rightKnee;
          LeftAnkle = leftAnkle; RightAnkle = rightAnkle; LeftToe = leftToe; RightToe = rightToe;
          Chest = chest; Neck = neck; Head = head; LeftShoulder = leftShoulder; RightShoulder = rightShoulder;
          LeftElbow = leftElbow; RightElbow = rightElbow; LeftHand = leftHand; RightHand = rightHand;
          PelvisForward = pelvisForward; PelvisUp = pelvisUp; ChestForward = chestForward;
          ChestUp = chestUp; HeadForward = headForward; }
    }

    public static class SkateRider
    {
        public const double MinimumMeasurement = 0.01;
        public const double MaximumMeasurement = 5;
        private const double ReachMargin = 0.000001;

        public static bool TryCreate(SkateRiderRig rig, SkateRiderInput input,
                                     out SkateRiderPose pose, out string error)
        {
            pose = default(SkateRiderPose);
            error = null;
            if (!Valid(rig) || !Valid(input))
            { error = "Invalid skate rider request."; return false; }

            var up = Unit(input.BoardUp);
            var forward = Unit(Project(input.BoardForward, up));
            var right = Cross(up, forward);
            int side = input.Stance == SkateStance.Regular ? 1 : -1;
            var facing = right * side;
            var travel = forward * (input.Switch ? -1 : 1);
            double lift = SkatePose.FootLift + (input.Airborne && !input.Bailed
                ? SkatePose.FlipFootLift * Math.Abs(Math.Sin((input.FlipDegrees % 360) * Math.PI / 360)) : 0);
            var deck = input.BoardPosition + up * lift;
            var leftAnkle = deck + forward * (side * SkatePose.FootOffset);
            var rightAnkle = deck - forward * (side * SkatePose.FootOffset);
            bool backIsLeft = (input.Stance == SkateStance.Goofy) != input.Switch;
            if (input.Pushing && !input.Airborne && !input.Bailed)
            {
                var back = PushFoot(input.BoardPosition, travel, right * side, up, input.PushPhase);
                if (backIsLeft) leftAnkle = back; else rightAnkle = back;
            }

            double lean = input.LeanDegrees * Math.PI / 180;
            var pelvisUp = up * Math.Cos(lean) + right * Math.Sin(lean);
            var pelvisForward = (right * Math.Cos(lean) - up * Math.Sin(lean)) * side;
            var pelvisRight = Cross(pelvisUp, pelvisForward);
            double crouch = input.Bailed ? Math.Max(0.85, input.Crouch) : input.Crouch;
            double speed = input.Speed / SkateMotion.MaximumGroundSpeed;
            var centre = Project((leftAnkle + rightAnkle) * 0.5 - input.BoardPosition, up) + input.BoardPosition;
            centre += right * (rig.PelvisHeight * Math.Sin(lean) * 0.45) + travel * (0.035 * speed);
            var leftHipBase = centre - pelvisRight * (rig.HipWidth * 0.5);
            var rightHipBase = centre + pelvisRight * (rig.HipWidth * 0.5);
            double minimum = SkatePose.FootLift, maximum = MaximumMeasurement * 2;
            if (!HeightRange(leftHipBase, leftAnkle, up, rig.ThighLength, rig.ShinLength, ref minimum, ref maximum) ||
                !HeightRange(rightHipBase, rightAnkle, up, rig.ThighLength, rig.ShinLength, ref minimum, ref maximum))
            { error = "Skate rider legs cannot reach the foot positions."; return false; }
            double requested = rig.PelvisHeight - Math.Min(SkatePose.CrouchDrop, rig.PelvisHeight * 0.65) * crouch;
            double height = Math.Max(minimum, Math.Min(maximum, requested));
            var pelvis = centre + up * height;
            var leftHip = leftHipBase + up * height;
            var rightHip = rightHipBase + up * height;
            SkateVector leftKnee, rightKnee;
            if (!Limb(leftHip, leftAnkle, rig.ThighLength, rig.ShinLength,
                      facing - pelvisRight * 0.15, input.BoardPosition, up, out leftKnee) ||
                !Limb(rightHip, rightAnkle, rig.ThighLength, rig.ShinLength,
                      facing + pelvisRight * 0.15, input.BoardPosition, up, out rightKnee))
            { error = "Skate rider legs cannot reach above the ground plane."; return false; }

            double chestLean = input.Bailed ? 0.65 : lean * 0.65 * side;
            var chestUp = up * Math.Cos(chestLean) + facing * Math.Sin(chestLean);
            var chestForward = facing * Math.Cos(chestLean) - up * Math.Sin(chestLean);
            var chestRight = Cross(chestUp, chestForward);
            var chest = pelvis + chestUp * rig.SpineLength;
            var neck = chest + chestUp * (rig.SpineLength * 0.12);
            var head = neck + chestUp * rig.NeckToHeadLength;
            var leftShoulder = chest - chestRight * (rig.ShoulderWidth * 0.5);
            var rightShoulder = chest + chestRight * (rig.ShoulderWidth * 0.5);
            double armLength = rig.UpperArmLength + rig.ForearmLength;
            var armForward = chestForward * (armLength * (input.Bailed ? 0.5 : 0.1 + 0.05 * speed));
            var armDown = up * (-armLength * (input.Bailed ? 0.15 : 0.28));
            double spread = input.Bailed ? 0.22 : 0.62;
            var leftTarget = leftShoulder - chestRight * (armLength * spread) + armForward + armDown;
            var rightTarget = rightShoulder + chestRight * (armLength * spread) + armForward + armDown;
            if (input.Grab && input.Airborne && !input.Bailed)
            {
                var grab = input.BoardPosition + facing * 0.1 - travel * 0.12 + up * (lift + 0.05);
                if (backIsLeft) leftTarget = grab; else rightTarget = grab;
            }
            SkateVector leftHand, rightHand, leftElbow, rightElbow;
            if (!Arm(leftShoulder, leftTarget, rig, -1, chestRight, chestForward, input.BoardPosition, up,
                     out leftElbow, out leftHand) ||
                !Arm(rightShoulder, rightTarget, rig, 1, chestRight, chestForward, input.BoardPosition, up,
                     out rightElbow, out rightHand))
            { error = "Skate rider arms cannot reach above the ground plane."; return false; }
            var look = Forward(input.LookYawDegrees % 360);
            var headForward = Unit(Project(look, up));
            if (headForward.LengthSquared < 0.5) headForward = forward;
            pose = new SkateRiderPose(pelvis, leftHip, rightHip, leftKnee, rightKnee, leftAnkle, rightAnkle,
                leftAnkle + facing * rig.FootLength, rightAnkle + facing * rig.FootLength,
                chest, neck, head, leftShoulder, rightShoulder, leftElbow, rightElbow, leftHand, rightHand,
                pelvisForward, pelvisUp, chestForward, chestUp, headForward);
            if (!Finite(pose))
            { pose = default(SkateRiderPose); error = "Skate rider pose exceeds the allowed bounds."; return false; }
            return true;
        }

        private static SkateVector PushFoot(SkateVector origin, SkateVector travel, SkateVector side,
                                           SkateVector up, double phase)
        {
            var deck = origin - travel * SkatePose.FootOffset + up * SkatePose.FootLift;
            var front = origin + travel * 0.1 + side * SkatePose.PushFootSide + up * SkatePose.FootLift;
            var back = origin - travel * SkatePose.PushFootBack + side * SkatePose.PushFootSide + up * SkatePose.FootLift;
            var lifted = back + up * 0.1;
            if (phase <= 0.25) return Blend(deck, front, Smooth(phase / 0.25));
            if (phase <= 0.6) return Blend(front, back, Smooth((phase - 0.25) / 0.35));
            if (phase <= 0.85) return Blend(back, lifted, Smooth((phase - 0.6) / 0.25));
            return Blend(lifted, deck, Smooth((phase - 0.85) / 0.15));
        }

        private static bool HeightRange(SkateVector hip, SkateVector ankle, SkateVector up,
                                        double first, double second, ref double minimum, ref double maximum)
        {
            var offset = ankle - hip;
            double vertical = SkateVector.Dot(offset, up);
            double horizontal = Math.Max(0, offset.LengthSquared - vertical * vertical);
            double far = first + second - ReachMargin;
            double near = Math.Sqrt(Math.Abs(first * first - second * second)) + ReachMargin;
            if (horizontal > far * far) return false;
            minimum = Math.Max(minimum, vertical + Math.Sqrt(Math.Max(0, near * near - horizontal)));
            maximum = Math.Min(maximum, vertical + Math.Sqrt(Math.Max(0, far * far - horizontal)));
            return minimum <= maximum;
        }

        private static bool Arm(SkateVector shoulder, SkateVector target, SkateRiderRig rig, int side,
                                SkateVector right, SkateVector forward, SkateVector ground, SkateVector up,
                                out SkateVector elbow, out SkateVector hand)
        {
            var offset = target - shoulder;
            double distance = offset.Length;
            var direction = distance > ReachMargin ? offset * (1 / distance) : forward;
            double reach = Math.Max(Math.Abs(rig.UpperArmLength - rig.ForearmLength) + ReachMargin,
                Math.Min(rig.UpperArmLength + rig.ForearmLength - ReachMargin, distance));
            double floor = (SkatePose.FootLift - SkateVector.Dot(shoulder - ground, up)) / reach;
            double vertical = SkateVector.Dot(direction, up);
            if (vertical < floor)
            {
                var horizontal = Unit(Project(direction, up));
                if (horizontal.LengthSquared < 0.5) horizontal = right * side;
                vertical = Math.Max(-1, Math.Min(1, floor));
                direction = horizontal * Math.Sqrt(Math.Max(0, 1 - vertical * vertical)) + up * vertical;
            }
            hand = shoulder + direction * reach;
            var bend = forward * -1 + right * (side * 0.35) + up * 0.25;
            if (Limb(shoulder, hand, rig.UpperArmLength, rig.ForearmLength, bend, ground, up, out elbow)) return true;
            hand = shoulder + right * (side * reach);
            return Limb(shoulder, hand, rig.UpperArmLength, rig.ForearmLength, bend, ground, up, out elbow);
        }

        private static bool Limb(SkateVector root, SkateVector end, double first, double second,
                                 SkateVector bend, SkateVector ground, SkateVector up, out SkateVector joint)
        {
            joint = default(SkateVector);
            var offset = end - root;
            double distance = offset.Length;
            if (distance <= 1e-12 || distance > first + second + 1e-8 ||
                distance < Math.Abs(first - second) - 1e-8) return false;
            var direction = offset * (1 / distance);
            double along = (first * first - second * second + distance * distance) / (2 * distance);
            double radius = Math.Sqrt(Math.Max(0, first * first - along * along));
            var centre = root + direction * along;
            var pole = Unit(Project(bend, direction));
            if (pole.LengthSquared < 0.5)
            {
                pole = Unit(Project(up, direction));
                if (pole.LengthSquared < 0.5) pole = Unit(Cross(direction,
                    Math.Abs(direction.X) < 0.75 ? new SkateVector(1, 0, 0) : new SkateVector(0, 1, 0)));
            }
            double centreHeight = SkateVector.Dot(centre - ground, up);
            if (centreHeight + radius * SkateVector.Dot(pole, up) < 0)
            {
                var high = Project(up, direction);
                double heightRange = high.Length;
                if (radius <= 1e-12 || heightRange <= 1e-12)
                { joint = centre + pole * radius; return SkateVector.Dot(joint - ground, up) >= -1e-8; }
                double required = -centreHeight / (radius * heightRange);
                if (required > 1 + 1e-8) return false;
                high *= 1 / heightRange;
                var across = Unit(Project(pole, high));
                if (across.LengthSquared < 0.5) across = Unit(Cross(direction, high));
                required = Math.Max(-1, Math.Min(1, required));
                pole = high * required + across * Math.Sqrt(Math.Max(0, 1 - required * required));
            }
            joint = centre + pole * radius;
            return Vector(joint) && SkateVector.Dot(joint - ground, up) >= -1e-7;
        }

        private static bool Valid(SkateRiderRig rig)
        { return Measurement(rig.PelvisHeight) && Measurement(rig.HipWidth) && Measurement(rig.ThighLength) &&
            Measurement(rig.ShinLength) && Measurement(rig.FootLength) && Measurement(rig.SpineLength) &&
            Measurement(rig.ShoulderWidth) && Measurement(rig.UpperArmLength) && Measurement(rig.ForearmLength) &&
            Measurement(rig.NeckToHeadLength); }
        private static bool Valid(SkateRiderInput input)
        { return Vector(input.BoardPosition) && Math.Abs(input.BoardPosition.X) <= 100000 &&
            Math.Abs(input.BoardPosition.Y) <= 100000 && Math.Abs(input.BoardPosition.Z) <= 100000 &&
            Vector(input.BoardForward) && Vector(input.BoardUp) &&
            Math.Abs(input.BoardForward.LengthSquared - 1) <= 0.0001 && Math.Abs(input.BoardUp.LengthSquared - 1) <= 0.0001 &&
            Math.Abs(SkateVector.Dot(input.BoardForward, input.BoardUp)) <= 0.0001 && input.BoardUp.Y > 0 &&
            (input.Stance == SkateStance.Regular || input.Stance == SkateStance.Goofy) &&
            Finite(input.Speed) && input.Speed >= 0 && input.Speed <= SkateMotion.MaximumGroundSpeed &&
            Finite(input.LeanDegrees) && Math.Abs(input.LeanDegrees) <= SkatePose.MaximumLean &&
            Fraction(input.Crouch) && Fraction(input.PushPhase) && Finite(input.FlipDegrees) &&
            Math.Abs(input.FlipDegrees) <= 3600000 && Finite(input.LookYawDegrees) && Math.Abs(input.LookYawDegrees) <= 3600000; }
        private static bool Finite(SkateRiderPose p)
        { return Vector(p.Pelvis) && Vector(p.LeftHip) && Vector(p.RightHip) && Vector(p.LeftKnee) && Vector(p.RightKnee) &&
            Vector(p.LeftAnkle) && Vector(p.RightAnkle) && Vector(p.LeftToe) && Vector(p.RightToe) && Vector(p.Chest) &&
            Vector(p.Neck) && Vector(p.Head) && Vector(p.LeftShoulder) && Vector(p.RightShoulder) &&
            Vector(p.LeftElbow) && Vector(p.RightElbow) && Vector(p.LeftHand) && Vector(p.RightHand) &&
            UnitVector(p.PelvisForward) && UnitVector(p.PelvisUp) && UnitVector(p.ChestForward) &&
            UnitVector(p.ChestUp) && UnitVector(p.HeadForward); }
        private static bool Measurement(double value)
        { return Finite(value) && value >= MinimumMeasurement && value <= MaximumMeasurement; }
        private static bool Fraction(double value) { return Finite(value) && value >= 0 && value <= 1; }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
        private static bool Vector(SkateVector value) { return Finite(value.X) && Finite(value.Y) && Finite(value.Z); }
        private static bool UnitVector(SkateVector value) { return Vector(value) && Math.Abs(value.LengthSquared - 1) < 1e-6; }
        private static SkateVector Unit(SkateVector value)
        { double length = value.Length; return length > 1e-12 ? value * (1 / length) : default(SkateVector); }
        private static SkateVector Project(SkateVector value, SkateVector normal) { return value - normal * SkateVector.Dot(value, normal); }
        private static SkateVector Cross(SkateVector a, SkateVector b)
        { return new SkateVector(a.Y * b.Z - a.Z * b.Y, a.Z * b.X - a.X * b.Z, a.X * b.Y - a.Y * b.X); }
        private static SkateVector Forward(double yaw)
        { double radians = yaw * Math.PI / 180; return new SkateVector(Math.Sin(radians), 0, Math.Cos(radians)); }
        private static SkateVector Blend(SkateVector a, SkateVector b, double t) { return a + (b - a) * t; }
        private static double Smooth(double t) { return t * t * (3 - 2 * t); }
    }
}
