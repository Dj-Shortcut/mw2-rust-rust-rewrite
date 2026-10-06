using System;

namespace Shortcut.RustMod
{
    public struct SkateRail
    {
        public readonly long Id;
        public readonly long Revision;
        public readonly SkateVector Start;
        public readonly SkateVector End;
        public SkateRail(long id, long revision, SkateVector start, SkateVector end)
        { Id = id; Revision = revision; Start = start; End = end; }
        public double Length { get { return (End - Start).Length; } }
        public SkateVector Direction { get { return (End - Start) * (1 / Length); } }
    }

    public sealed class SkateRailSet
    {
        public const int MaximumCount = 64;
        public static readonly SkateRailSet Empty = new SkateRailSet(new SkateRail[0]);
        private readonly SkateRail[] rails;
        private SkateRailSet(SkateRail[] rails) { this.rails = rails; }
        public int Count { get { return rails.Length; } }

        public static bool TryCreate(SkateRail[] definitions, out SkateRailSet result, out string error)
        {
            result = null;
            error = null;
            if (definitions == null || definitions.Length > MaximumCount)
            { error = "Invalid rail catalog size."; return false; }
            var copy = (SkateRail[])definitions.Clone();
            for (int i = 0; i < copy.Length; i++)
            {
                if (!ValidRail(copy[i])) { error = "Invalid rail definition."; return false; }
                for (int j = 0; j < i; j++)
                    if (copy[i].Id == copy[j].Id) { error = "Duplicate rail identity."; return false; }
            }
            result = new SkateRailSet(copy);
            return true;
        }

        public bool TryGet(long id, out SkateRail rail)
        {
            for (int i = 0; i < rails.Length; i++)
                if (rails[i].Id == id) { rail = rails[i]; return true; }
            rail = default(SkateRail);
            return false;
        }

        internal static bool ValidRail(SkateRail rail)
        {
            return rail.Id > 0 && rail.Revision > 0 && Coordinate(rail.Start) && Coordinate(rail.End) &&
                rail.Start.Y == rail.End.Y && rail.Length >= 0.25 && rail.Length <= 100;
        }
        internal static bool Same(SkateRail a, SkateRail b)
        {
            return a.Id == b.Id && a.Revision == b.Revision &&
                a.Start.X == b.Start.X && a.Start.Y == b.Start.Y && a.Start.Z == b.Start.Z &&
                a.End.X == b.End.X && a.End.Y == b.End.Y && a.End.Z == b.End.Z;
        }
        private static bool Coordinate(SkateVector value)
        { return Finite(value.X) && Finite(value.Y) && Finite(value.Z) &&
            Math.Abs(value.X) <= 100000 && Math.Abs(value.Y) <= 100000 && Math.Abs(value.Z) <= 100000; }
        private static bool Finite(double value) { return !double.IsNaN(value) && !double.IsInfinity(value); }
    }
}
