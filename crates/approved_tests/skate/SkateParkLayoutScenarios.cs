using System;
using System.Globalization;
using System.IO;
using System.Security.Cryptography;
using Shortcut.RustMod;

internal static class SkateParkLayoutScenarios
{
    private const double Tolerance = 1e-10;
    private static int checks;

    private struct ExpectedPart
    {
        public readonly double Right, Forward, Height, Pitch;
        public ExpectedPart(double right, double forward, double height, double pitch)
        { Right = right; Forward = forward; Height = height; Pitch = pitch; }
    }

    private sealed class Piece
    {
        public readonly SkateParkKind Kind;
        public readonly string Name;
        public readonly ExpectedPart[] Parts;
        public Piece(SkateParkKind kind, string name, params ExpectedPart[] parts)
        { Kind = kind; Name = name; Parts = parts; }
    }

    private struct Point
    {
        public double X, Y, Z;
        public Point(double x, double y, double z) { X = x; Y = y; Z = z; }
    }

    private struct Angle
    {
        public readonly double Input, Normalized, Cos, Sin;
        public Angle(double input, double normalized, double cos, double sin)
        { Input = input; Normalized = normalized; Cos = cos; Sin = sin; }
    }

    private static readonly Piece[] Pieces =
    {
        new Piece(SkateParkKind.Bank, "bank", new ExpectedPart(0, 0, 0.311867536226639, -12)),
        new Piece(SkateParkKind.Kicker, "kicker", new ExpectedPart(0, 0, 0.6339273926110491, -25)),
        new Piece(SkateParkKind.QuarterPipe, "quarterpipe",
            new ExpectedPart(0, 0, 0.3882285676537811, -15),
            new ExpectedPart(0, 2.67761680586709, 1.6368217898341313, -35)),
        new Piece(SkateParkKind.LedgeLow, "ledge-low", new ExpectedPart(0, 0, 0.4, 0)),
        new Piece(SkateParkKind.LedgeHigh, "ledge-high", new ExpectedPart(0, 0, 0.8, 0)),
        new Piece(SkateParkKind.LongLedge, "long-ledge",
            new ExpectedPart(0, 0, 0.4, 0), new ExpectedPart(-3, 0, 0.4, 0), new ExpectedPart(3, 0, 0.4, 0)),
        new Piece(SkateParkKind.Funbox, "funbox",
            new ExpectedPart(0, 0, 0.7764571353075622, 0),
            new ExpectedPart(0, -2.9488887394336025, 0.3882285676537811, -15),
            new ExpectedPart(0, 2.9488887394336025, 0.3882285676537811, 15))
    };

    private static int Main(string[] args)
    {
        try
        {
            string path = typeof(SkateParkLayout).Assembly.Location;
            string hash;
            using (FileStream stream = File.OpenRead(path))
            using (SHA256 sha = SHA256.Create())
                hash = BitConverter.ToString(sha.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
            Console.WriteLine("Plugin: " + path);
            Console.WriteLine("SHA-256: " + hash);
            if (args.Length > 1 || (args.Length == 1 && !string.Equals(args[0], hash, StringComparison.OrdinalIgnoreCase)))
                throw new InvalidOperationException("Expected one optional SHA-256 matching the loaded plugin DLL.");
            PieceLayouts();
            AnchorYawAndSnap();
            Console.WriteLine("PASS: both approved park layout scenarios (" + checks.ToString(CultureInfo.InvariantCulture) + " checks).");
            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine(error.Message);
            return 1;
        }
    }

    private static void PieceLayouts()
    {
        int start = checks;
        foreach (Piece piece in Pieces)
        {
            Require(SkateParkLayout.Name(piece.Kind) == piece.Name, piece.Name + ": canonical name.");
            SkateParkKind parsed;
            Require(SkateParkLayout.TryKind(piece.Name.ToUpperInvariant(), out parsed) && parsed == piece.Kind,
                    piece.Name + ": command name does not select the requested piece.");
            Require(SkateParkLayout.PartCount(piece.Kind) == piece.Parts.Length, piece.Name + ": floor count.");
            for (int i = 0; i < piece.Parts.Length; i++)
            {
                SkateParkPart part;
                Require(SkateParkLayout.TryPart(piece.Kind, i, out part), piece.Name + ": missing floor " + i + ".");
                ExpectedPart expected = piece.Parts[i];
                Near(part.Right, expected.Right, piece.Name + ": right.");
                Near(part.Forward, expected.Forward, piece.Name + ": forward.");
                Near(part.Height, expected.Height, piece.Name + ": height.");
                Near(part.Pitch, expected.Pitch, piece.Name + ": pitch.");
            }
        }

        Point bankEntry = LocalEdge(SkateParkKind.Bank, 0, 0, -1.5);
        Point bankExit = LocalEdge(SkateParkKind.Bank, 0, 0, 1.5);
        Point kickerEntry = LocalEdge(SkateParkKind.Kicker, 0, 0, -1.5);
        Point kickerExit = LocalEdge(SkateParkKind.Kicker, 0, 0, 1.5);
        Near(bankEntry.Y, 0, "bank: entrance meets anchor plane.");
        Near(bankExit.Y, 0.623735072453278, "bank: exit rise.");
        Near(kickerEntry.Y, 0, "kicker: entrance meets anchor plane.");
        Near(kickerExit.Y, 1.2678547852220982, "kicker: exit rise.");
        Near(LocalEdge(SkateParkKind.QuarterPipe, 0, 0, -1.5).Y, 0, "quarterpipe: entrance.");
        Near(LocalEdge(SkateParkKind.QuarterPipe, 1, 0, 1.5).Y, 2.4971864443607004, "quarterpipe: exit rise.");
        Near(LocalEdge(SkateParkKind.QuarterPipe, 1, 0, 1.5).Z -
             LocalEdge(SkateParkKind.QuarterPipe, 0, 0, -1.5).Z,
             5.35523361173418, "quarterpipe: projected run.");
        Near(LocalEdge(SkateParkKind.Funbox, 1, 0, -1.5).Y, 0, "funbox: leading entrance.");
        Near(LocalEdge(SkateParkKind.Funbox, 2, 0, 1.5).Y, 0, "funbox: trailing entrance.");
        Near(LocalEdge(SkateParkKind.Funbox, 2, 0, 1.5).Z -
             LocalEdge(SkateParkKind.Funbox, 1, 0, -1.5).Z,
             8.79555495773441, "funbox: projected run.");
        Near(LocalEdge(SkateParkKind.LongLedge, 2, 1.5, 0).X -
             LocalEdge(SkateParkKind.LongLedge, 1, -1.5, 0).X, 9, "long-ledge: total width.");
        CheckSeams(0, 0, 0, 0);
        Console.WriteLine("PASS: skate_park_piece_layouts (7 pieces, 12 floors, " +
                          (checks - start).ToString(CultureInfo.InvariantCulture) + " checks).");
    }

    private static void AnchorYawAndSnap()
    {
        int start = checks;
        Angle[] angles =
        {
            new Angle(0, 0, 1, 0), new Angle(90, 90, 0, 1), new Angle(180, 180, -1, 0),
            new Angle(270, 270, 0, -1), new Angle(-90, 270, 0, -1), new Angle(450, 90, 0, 1),
            new Angle(720, 0, 1, 0), new Angle(-720, 0, 1, 0),
            new Angle(37.25, 37.25, 0.796002002534622, 0.6052939880428944),
            new Angle(123.75, 123.75, -0.5555702330196023, 0.8314696123025451)
        };
        Point[] anchors = { new Point(17.375, 2.625, -8.0625), new Point(-23.8125, -4.375, 31.1875) };
        Point translation = new Point(0.34375, 1.125, -0.21875);
        foreach (Point anchor in anchors)
        foreach (Angle angle in angles)
        {
            foreach (Piece piece in Pieces)
            for (int i = 0; i < piece.Parts.Length; i++)
            {
                ExpectedPart expected = piece.Parts[i];
                SkateParkTransform value = Transform(piece.Kind, i, anchor.X, anchor.Y, anchor.Z, angle.Input);
                Near(value.X, anchor.X + expected.Right * angle.Cos + expected.Forward * angle.Sin, piece.Name + ": rotated X.");
                Near(value.Y, anchor.Y + expected.Height, piece.Name + ": translated Y.");
                Near(value.Z, anchor.Z - expected.Right * angle.Sin + expected.Forward * angle.Cos, piece.Name + ": rotated Z.");
                Near(value.Yaw, angle.Normalized, piece.Name + ": normalized yaw.");
                Near(value.Pitch, expected.Pitch, piece.Name + ": authored pitch preserved.");
                Near((value.X - anchor.X) * angle.Cos - (value.Z - anchor.Z) * angle.Sin,
                     expected.Right, piece.Name + ": recovered local right.");
                Near((value.X - anchor.X) * angle.Sin + (value.Z - anchor.Z) * angle.Cos,
                     expected.Forward, piece.Name + ": recovered local forward.");
                SkateParkTransform shifted = Transform(piece.Kind, i, anchor.X + translation.X,
                    anchor.Y + translation.Y, anchor.Z + translation.Z, angle.Input);
                Near(shifted.X - value.X, translation.X, piece.Name + ": fractional translation X.");
                Near(shifted.Y - value.Y, translation.Y, piece.Name + ": fractional translation Y.");
                Near(shifted.Z - value.Z, translation.Z, piece.Name + ": fractional translation Z.");
            }
            CheckSeams(anchor.X, anchor.Y, anchor.Z, angle.Input);
        }
        Console.WriteLine("PASS: skate_park_anchor_yaw_and_snap (20 translated/yaw anchors, " +
                          (checks - start).ToString(CultureInfo.InvariantCulture) + " checks).");
    }

    private static void CheckSeams(double x, double y, double z, double yaw)
    {
        foreach (double side in new[] { -1.5, 1.5 })
        {
            Same(WorldEdge(SkateParkKind.QuarterPipe, 0, side, 1.5, x, y, z, yaw),
                 WorldEdge(SkateParkKind.QuarterPipe, 1, side, -1.5, x, y, z, yaw), "quarterpipe: joined slope corners.");
            Same(WorldEdge(SkateParkKind.Funbox, 1, side, 1.5, x, y, z, yaw),
                 WorldEdge(SkateParkKind.Funbox, 0, side, -1.5, x, y, z, yaw), "funbox: leading deck corners.");
            Same(WorldEdge(SkateParkKind.Funbox, 2, side, -1.5, x, y, z, yaw),
                 WorldEdge(SkateParkKind.Funbox, 0, side, 1.5, x, y, z, yaw), "funbox: trailing deck corners.");
            Same(WorldEdge(SkateParkKind.LongLedge, 1, 1.5, side, x, y, z, yaw),
                 WorldEdge(SkateParkKind.LongLedge, 0, -1.5, side, x, y, z, yaw), "long-ledge: left seam corners.");
            Same(WorldEdge(SkateParkKind.LongLedge, 2, -1.5, side, x, y, z, yaw),
                 WorldEdge(SkateParkKind.LongLedge, 0, 1.5, side, x, y, z, yaw), "long-ledge: right seam corners.");
        }
    }

    private static Point LocalEdge(SkateParkKind kind, int index, double right, double forward)
    { return WorldEdge(kind, index, right, forward, 0, 0, 0, 0); }

    private static Point WorldEdge(SkateParkKind kind, int index, double right, double forward,
                                   double x, double y, double z, double yaw)
    {
        SkateParkTransform value = Transform(kind, index, x, y, z, yaw);
        double pitch = value.Pitch * Math.PI / 180, heading = value.Yaw * Math.PI / 180;
        double horizontal = forward * Math.Cos(pitch);
        return new Point(value.X + right * Math.Cos(heading) + horizontal * Math.Sin(heading),
                         value.Y - forward * Math.Sin(pitch),
                         value.Z - right * Math.Sin(heading) + horizontal * Math.Cos(heading));
    }

    private static SkateParkTransform Transform(SkateParkKind kind, int index, double x, double y, double z, double yaw)
    {
        SkateParkTransform value;
        Require(SkateParkLayout.TryTransform(kind, index, x, y, z, yaw, out value), kind + ": valid anchor refused.");
        return value;
    }

    private static void Same(Point first, Point second, string message)
    { Near(first.X, second.X, message + " X"); Near(first.Y, second.Y, message + " Y"); Near(first.Z, second.Z, message + " Z"); }

    private static void Near(double actual, double expected, string message)
    { Require(!double.IsNaN(actual) && !double.IsInfinity(actual) && Math.Abs(actual - expected) <= Tolerance, message); }

    private static void Require(bool condition, string message)
    { checks++; if (!condition) throw new InvalidOperationException(message); }
}
