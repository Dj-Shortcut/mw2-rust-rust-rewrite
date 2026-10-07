using System;

namespace Shortcut.RustMod
{
    public sealed class SkateWorldSweep
    {
        public readonly SkateVector Center;
        public readonly SkateVector HalfExtents;
        public readonly SkateVector Direction;
        public readonly double Distance;
        public readonly bool Mounted;
        internal readonly SkateVector OriginalDelta;

        internal SkateWorldSweep(SkateVector center, SkateVector extents, SkateVector direction, double distance, bool mounted, SkateVector originalDelta)
        { Center = center; HalfExtents = extents; Direction = direction; Distance = distance; Mounted = mounted; OriginalDelta = originalDelta; }
    }

    public struct SkateWorldContact
    {
        public readonly double Distance;
        public readonly SkateVector Normal;
        public readonly bool Self;
        public readonly long RailId;

        public SkateWorldContact(double distance, SkateVector normal, bool self)
            : this(distance, normal, self, 0) { }

        // RailId comes only from a trusted host binding of the actual hit collider; zero is ordinary.
        public SkateWorldContact(double distance, SkateVector normal, bool self, long railId)
        { Distance = distance; Normal = normal; Self = self; RailId = railId; }
    }

    public static class SkateWorldQuery
    {
        public const int Capacity = 128;
        public const double MaximumDistance = 16;
        private const double Precision = SkateMotion.Skin / 8;

        public static bool TryPrepare(SkateVector start, SkateVector end, SkateHull hull, out SkateWorldSweep query)
        {
            query = null;
            bool mounted = SameHull(hull, SkateMotion.MountedHull);
            if (!mounted && !SameHull(hull, SkateMotion.StandingHull)) return false;
            SkateVector a, b, center, destination, extents;
            if (!TryVector(start, out a) || !TryVector(end, out b) ||
                !TryVector(start + hull.CentreOffset, out center) ||
                !TryVector(end + hull.CentreOffset, out destination) ||
                !TryVector(hull.HalfExtents, out extents)) return false;
            double originalLength = (end - start).Length;
            SkateVector delta = destination - center;
            double length = delta.Length;
            if (!Finite(originalLength) || originalLength > MaximumDistance ||
                !Finite(length) || length > MaximumDistance) return false;
            // Start and end may round to the same native point, as in a rail seat snap; that sub-precision
            // move is checked as a zero-distance overlap. Any larger move collapsing to zero is refused.
            if (originalLength > Precision && length == 0) return false;
            SkateVector direction = default(SkateVector);
            double distance;
            if (!TryScalar(length, out distance)) return false;
            if (length > 0 && !TryVector(delta * (1 / length), out direction)) return false;
            query = new SkateWorldSweep(center, extents, direction, distance, mounted, end - start);
            return true;
        }

        public static bool TrySelect(SkateWorldSweep query, SkateWorldContact[] contacts, int count, out SkateHit hit)
        {
            hit = SkateHit.Miss;
            if (query == null || contacts == null || contacts.Length != Capacity || count < 0 || count >= Capacity) return false;
            bool found = false;
            double nearest = double.MaxValue;
            SkateVector normal = default(SkateVector);
            long railId = 0;
            bool ambiguous = false;
            for (int i = 0; i < count; ++i)
            {
                SkateWorldContact contact = contacts[i];
                double squared = contact.Normal.LengthSquared;
                if (!Finite(contact.Distance) || contact.Distance < 0 || contact.Distance > query.Distance ||
                    !Finite(squared) || Math.Abs(squared - 1) > 0.001 || contact.RailId < 0) return false;
                SkateVector unit = contact.Normal * (1 / Math.Sqrt(squared));
                double entering = SkateVector.Dot(unit, query.Direction);
                if (!Finite(entering)) return false;
                if (contact.Self) continue;
                if (query.Distance == 0 || entering >= -0.000001)
                {
                    if (contact.Distance == 0) continue;
                    return false;
                }
                if (!found || contact.Distance < nearest)
                {
                    found = true; nearest = contact.Distance; normal = unit;
                    railId = contact.RailId; ambiguous = false;
                    continue;
                }
                if (contact.Distance != nearest) continue;
                // Equal-distance blockers with different identities are an ordinary contact.
                if (contact.RailId != railId) ambiguous = true;
                if (Earlier(unit, normal)) normal = unit;
            }
            if (found)
            {
                if (SkateVector.Dot(query.OriginalDelta, normal) > 0.000001) return false;
                hit = new SkateHit(true, false, nearest / query.Distance, normal, ambiguous ? 0 : railId);
            }
            return true;
        }

        public static bool SameHull(SkateHull a, SkateHull b)
        { return Same(a.HalfExtents, b.HalfExtents) && Same(a.CentreOffset, b.CentreOffset); }

        private static bool Same(SkateVector a, SkateVector b)
        { return a.X == b.X && a.Y == b.Y && a.Z == b.Z; }

        private static bool Earlier(SkateVector a, SkateVector b)
        { return a.X < b.X || (a.X == b.X && (a.Y < b.Y || (a.Y == b.Y && a.Z < b.Z))); }

        private static bool TryVector(SkateVector value, out SkateVector converted)
        {
            converted = default(SkateVector);
            double x, y, z;
            if (!TryScalar(value.X, out x) || !TryScalar(value.Y, out y) || !TryScalar(value.Z, out z)) return false;
            converted = new SkateVector(x, y, z);
            return true;
        }

        private static bool TryScalar(double value, out double converted)
        {
            converted = 0;
            if (!Finite(value) || Math.Abs(value) > 4096) return false;
            float single = (float)value;
            int exponent = (BitConverter.ToInt32(BitConverter.GetBytes(single), 0) >> 23) & 255;
            double spacing = Math.Pow(2, exponent == 0 ? -149 : exponent - 150);
            if (spacing > Precision || Math.Abs(value - single) > Precision) return false;
            converted = single;
            return true;
        }

        private static bool Finite(double value)
        { return !double.IsNaN(value) && !double.IsInfinity(value); }
    }
}
