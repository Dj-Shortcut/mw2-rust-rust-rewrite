using System;
using System.Collections.Generic;

namespace Shortcut.RustMod
{
    public sealed class SkateBoardMesh
    {
        public readonly float[] Positions;
        public readonly float[] Normals;
        public readonly int[] Triangles;
        public readonly float[] Colours;

        private SkateBoardMesh(float[] positions, float[] normals, int[] triangles, float[] colours)
        { Positions = positions; Normals = normals; Triangles = triangles; Colours = colours; }

        public static SkateBoardMesh Create()
        {
            var builder = new Builder();
            builder.Deck();
            builder.Truck(-0.245f);
            builder.Truck(0.245f);
            return builder.Build();
        }

        private struct Point
        {
            public readonly float X, Y, Z;
            public Point(float x, float y, float z) { X = x; Y = y; Z = z; }
            public static Point operator -(Point a, Point b) { return new Point(a.X - b.X, a.Y - b.Y, a.Z - b.Z); }
            public static Point Cross(Point a, Point b)
            { return new Point(a.Y * b.Z - a.Z * b.Y, a.Z * b.X - a.X * b.Z, a.X * b.Y - a.Y * b.X); }
            public static float Dot(Point a, Point b) { return a.X * b.X + a.Y * b.Y + a.Z * b.Z; }
        }

        private struct Colour
        {
            public readonly float R, G, B;
            public Colour(float r, float g, float b) { R = r; G = g; B = b; }
        }

        private sealed class Builder
        {
            private readonly List<float> positions = new List<float>(4608);
            private readonly List<float> normals = new List<float>(4608);
            private readonly List<int> triangles = new List<int>(1536);
            private readonly List<float> colours = new List<float>(6144);
            private static readonly Colour Grip = new Colour(0.055f, 0.060f, 0.065f);
            private static readonly Colour Maple = new Colour(0.70f, 0.42f, 0.20f);
            private static readonly Colour Edge = new Colour(0.47f, 0.27f, 0.11f);
            private static readonly Colour Metal = new Colour(0.48f, 0.52f, 0.55f);
            private static readonly Colour Wheel = new Colour(0.80f, 0.82f, 0.76f);

            public SkateBoardMesh Build()
            { return new SkateBoardMesh(positions.ToArray(), normals.ToArray(), triangles.ToArray(), colours.ToArray()); }

            public void Deck()
            {
                float[] z = { -0.40f, -0.385f, -0.355f, -0.305f, -0.25f, 0, 0.25f, 0.305f, 0.355f, 0.385f, 0.40f };
                float[] width = { 0.02f, 0.055f, 0.085f, 0.095f, 0.10f, 0.10f, 0.10f, 0.095f, 0.085f, 0.055f, 0.02f };
                float[] height = { 0.142f, 0.134f, 0.120f, 0.102f, 0.092f, 0.092f, 0.092f, 0.102f, 0.120f, 0.134f, 0.142f };
                for (int i = 0; i < z.Length - 1; i++)
                {
                    var a = new Point(-width[i], height[i], z[i]);
                    var b = new Point(width[i], height[i], z[i]);
                    var c = new Point(width[i + 1], height[i + 1], z[i + 1]);
                    var d = new Point(-width[i + 1], height[i + 1], z[i + 1]);
                    var e = new Point(a.X, a.Y - 0.009f, a.Z);
                    var f = new Point(b.X, b.Y - 0.009f, b.Z);
                    var g = new Point(c.X, c.Y - 0.009f, c.Z);
                    var h = new Point(d.X, d.Y - 0.009f, d.Z);
                    Quad(a, b, c, d, new Point(0, 1, 0), Grip);
                    Quad(e, f, g, h, new Point(0, -1, 0), Maple);
                    Quad(a, d, h, e, new Point(-1, 0, 0), Edge);
                    Quad(b, c, g, f, new Point(1, 0, 0), Edge);
                    if (i == 0) Quad(a, b, f, e, new Point(0, 0, -1), Edge);
                    if (i == z.Length - 2) Quad(d, c, g, h, new Point(0, 0, 1), Edge);
                }
            }

            public void Truck(float z)
            {
                Box(new Point(0, 0.079f, z), new Point(0.0325f, 0.004f, 0.0275f), Metal);
                Box(new Point(0, 0.055f, z), new Point(0.0085f, 0.020f, 0.010f), Metal);
                Box(new Point(0, 0.033f, z), new Point(0.070f, 0.007f, 0.0125f), Metal);
                Cylinder(0, 0.0275f, z, 0.004f, 0.176f, 12, Metal);
                Cylinder(-0.090f, 0.0275f, z, 0.0275f, 0.024f, 16, Wheel);
                Cylinder(0.090f, 0.0275f, z, 0.0275f, 0.024f, 16, Wheel);
            }

            private void Box(Point centre, Point half, Colour colour)
            {
                var a = new Point(centre.X - half.X, centre.Y - half.Y, centre.Z - half.Z);
                var b = new Point(centre.X + half.X, centre.Y - half.Y, centre.Z - half.Z);
                var c = new Point(centre.X + half.X, centre.Y - half.Y, centre.Z + half.Z);
                var d = new Point(centre.X - half.X, centre.Y - half.Y, centre.Z + half.Z);
                var e = new Point(a.X, centre.Y + half.Y, a.Z);
                var f = new Point(b.X, centre.Y + half.Y, b.Z);
                var g = new Point(c.X, centre.Y + half.Y, c.Z);
                var h = new Point(d.X, centre.Y + half.Y, d.Z);
                Quad(a, b, c, d, new Point(0, -1, 0), colour);
                Quad(e, f, g, h, new Point(0, 1, 0), colour);
                Quad(a, b, f, e, new Point(0, 0, -1), colour);
                Quad(d, c, g, h, new Point(0, 0, 1), colour);
                Quad(a, d, h, e, new Point(-1, 0, 0), colour);
                Quad(b, c, g, f, new Point(1, 0, 0), colour);
            }

            private void Cylinder(float x, float y, float z, float radius, float length, int segments, Colour colour)
            {
                var left = new Point(x - length * 0.5f, y, z);
                var right = new Point(x + length * 0.5f, y, z);
                for (int i = 0; i < segments; i++)
                {
                    double angle = i * Math.PI * 2 / segments;
                    double next = (i + 1) % segments * Math.PI * 2 / segments;
                    float ay = radius * (float)Math.Cos(angle), az = radius * (float)Math.Sin(angle);
                    float by = radius * (float)Math.Cos(next), bz = radius * (float)Math.Sin(next);
                    var a = new Point(left.X, y + ay, z + az);
                    var b = new Point(left.X, y + by, z + bz);
                    var c = new Point(right.X, y + by, z + bz);
                    var d = new Point(right.X, y + ay, z + az);
                    Quad(a, b, c, d, new Point(0, ay + by, az + bz), colour);
                    Triangle(left, b, a, new Point(-1, 0, 0), colour);
                    Triangle(right, c, d, new Point(1, 0, 0), colour);
                }
            }

            private void Quad(Point a, Point b, Point c, Point d, Point outward, Colour colour)
            { Triangle(a, b, c, outward, colour); Triangle(a, c, d, outward, colour); }

            private void Triangle(Point a, Point b, Point c, Point outward, Colour colour)
            {
                var normal = Point.Cross(b - a, c - a);
                if (Point.Dot(normal, outward) < 0)
                {
                    var swap = b; b = c; c = swap;
                    normal = Point.Cross(b - a, c - a);
                }
                float length = (float)Math.Sqrt(Point.Dot(normal, normal));
                normal = new Point(normal.X / length, normal.Y / length, normal.Z / length);
                Vertex(a, normal, colour);
                Vertex(b, normal, colour);
                Vertex(c, normal, colour);
            }

            private void Vertex(Point point, Point normal, Colour colour)
            {
                triangles.Add(positions.Count / 3);
                positions.Add(point.X); positions.Add(point.Y); positions.Add(point.Z);
                normals.Add(normal.X); normals.Add(normal.Y); normals.Add(normal.Z);
                colours.Add(colour.R); colours.Add(colour.G); colours.Add(colour.B); colours.Add(1);
            }
        }
    }
}
