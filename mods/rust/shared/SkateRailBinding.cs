using System;
using System.Collections.Generic;

namespace Shortcut.RustMod
{
    // One host-registered native collider and the rail its top surface supports.
    public struct SkateRailCollider
    {
        public readonly long Key;
        public readonly long RailId;
        public SkateRailCollider(long key, long railId) { Key = key; RailId = railId; }
    }

    // Immutable server snapshot: the world query resolves hit colliders through the same
    // binding whose Rails catalog is passed to SkateMotion.TryStep. Changes need a new snapshot.
    public sealed class SkateRailBinding
    {
        public const int MaximumColliders = 128;
        public const double MaximumRailWidth = 0.6;
        public static readonly SkateRailBinding Empty =
            new SkateRailBinding(SkateRailSet.Empty, new Dictionary<long, long>());
        private readonly Dictionary<long, long> railByCollider;

        private SkateRailBinding(SkateRailSet rails, Dictionary<long, long> railByCollider)
        { Rails = rails; this.railByCollider = railByCollider; }

        public SkateRailSet Rails { get; private set; }
        public int ColliderCount { get { return railByCollider.Count; } }

        public static bool TryCreate(SkateRail[] rails, SkateRailCollider[] colliders,
                                     out SkateRailBinding binding, out string error)
        {
            binding = null;
            SkateRailSet set;
            if (!SkateRailSet.TryCreate(rails, out set, out error)) return false;
            if (colliders == null || colliders.Length > MaximumColliders)
                return Fail("Invalid rail collider count.", out error);
            var map = new Dictionary<long, long>(colliders.Length);
            var bound = new Dictionary<long, bool>();
            for (int i = 0; i < colliders.Length; i++)
            {
                SkateRailCollider collider = colliders[i];
                SkateRail rail;
                if (collider.Key == 0 || collider.RailId <= 0 || !set.TryGet(collider.RailId, out rail))
                    return Fail("Invalid rail collider binding.", out error);
                if (map.ContainsKey(collider.Key)) return Fail("Duplicate rail collider.", out error);
                map.Add(collider.Key, collider.RailId);
                bound[collider.RailId] = true;
            }
            // A rail without a collider could never report support, so it indicates a host error.
            if (bound.Count != set.Count) return Fail("Rail has no bound collider.", out error);
            binding = new SkateRailBinding(set, map);
            return true;
        }

        // Unbound or zero keys are ordinary world contacts.
        public long Resolve(long colliderKey)
        {
            long railId;
            return colliderKey != 0 && railByCollider.TryGetValue(colliderKey, out railId) ? railId : 0;
        }

        // Top centreline of a level box from its world centre, scaled local half extents and yaw
        // about +Y (zero faces +Z). The host must verify that the collider has no pitch or roll.
        public static bool TryRailFromBox(long id, long revision, SkateVector centre, SkateVector halfExtents,
                                          double yawDegrees, out SkateRail rail, out string error)
        {
            rail = default(SkateRail);
            error = null;
            if (!Finite(centre) || !Finite(halfExtents) || !Finite(yawDegrees) || Math.Abs(yawDegrees) > 720 ||
                halfExtents.X <= 0 || halfExtents.Y <= 0 || halfExtents.Z <= 0)
                return Fail("Invalid rail box.", out error);
            bool alongZ = halfExtents.Z > halfExtents.X;
            double half = alongZ ? halfExtents.Z : halfExtents.X;
            double width = alongZ ? halfExtents.X : halfExtents.Z;
            if (half == width || width * 2 > MaximumRailWidth) return Fail("Rail box is not a rail.", out error);
            double radians = yawDegrees * Math.PI / 180;
            double sin = Math.Sin(radians), cos = Math.Cos(radians);
            var axis = alongZ ? new SkateVector(sin, 0, cos) : new SkateVector(cos, 0, -sin);
            var top = new SkateVector(centre.X, centre.Y + halfExtents.Y, centre.Z);
            var candidate = new SkateRail(id, revision, top - axis * half, top + axis * half);
            if (!SkateRailSet.ValidRail(candidate)) return Fail("Invalid rail definition.", out error);
            rail = candidate;
            return true;
        }

        private static bool Finite(SkateVector value) { return Finite(value.X) && Finite(value.Y) && Finite(value.Z); }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
        private static bool Fail(string message, out string error) { error = message; return false; }
    }
}
