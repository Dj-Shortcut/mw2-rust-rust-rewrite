// Grinding. While the rider comes down through the air, the heights around them are sampled with
// downward rays and the shared edge module looks for a level ledge in that grid. The module says
// whether the approach is eligible; this adapter adds what it cannot know: that the rider is
// actually at the edge (close to its line, feet at its height) and that it carries on ahead.
// A captured rider is never moved onto the edge: the ride steers the body onto the line.
using System;
using Shortcut.RustMod;
using UnityEngine;

public static class SkateGrind
{
    public const int Rows = 9, Cols = 11;
    public const float RowStep = 0.2f, ColStep = 0.1f, Inset = 0.1f, Reach = 0.45f;
    public static bool Enabled = true;
    public static float Heading, Speed;
    public static Vector3 Target;
    public static int Scans, Finds, Captures;
    public static string Last = "";
    private static readonly double[] heights = new double[Rows * Cols];
    private static SkateEdge edge;
    private static double cooldown;
    private static bool failed;

    public static void Reset() { cooldown = 0; edge = default(SkateEdge); Speed = 0f; Last = ""; }

    private static float YawOf(double x, double z) { return (float)(Math.Atan2(x, z) * 180.0 / Math.PI); }

    // A grid centred on the rider: rows run along the heading, columns to its right.
    private static SkateHeightGrid Sample(Vector3 pos, float yaw)
    {
        var rad = yaw * Math.PI / 180.0;
        double fx = Math.Sin(rad), fz = Math.Cos(rad), rx = fz, rz = -fx;
        double ox = pos.x - fx * ((Rows - 1) * 0.5 * RowStep) - rx * ((Cols - 1) * 0.5 * ColStep);
        double oz = pos.z - fz * ((Rows - 1) * 0.5 * RowStep) - rz * ((Cols - 1) * 0.5 * ColStep);
        var top = pos.y + 0.7f;
        RaycastHit hit;
        for (var row = 0; row < Rows; row++)
            for (var col = 0; col < Cols; col++)
            {
                var from = new Vector3((float)(ox + fx * row * RowStep + rx * col * ColStep), top, (float)(oz + fz * row * RowStep + rz * col * ColStep));
                heights[row * Cols + col] = Physics.Raycast(from, Vector3.down, out hit, 2.3f, SkateRide.GroundMask, QueryTriggerInteraction.Ignore) ? hit.m_Point.y : double.NaN;
            }
        Scans++;
        return new SkateHeightGrid(new SkateVector(ox, pos.y, oz), new SkateVector(fx, 0, fz), new SkateVector(rx, 0, rz), RowStep, ColStep, Rows, Cols, heights);
    }

    // The point of the edge nearest the rider, a little onto the high side, and how far the rider is from the line.
    private static float Aim(Vector3 pos)
    {
        double dx = pos.x - edge.Point.X, dz = pos.z - edge.Point.Z;
        var along = dx * edge.Direction.X + dz * edge.Direction.Z;
        if (along > edge.Ahead) along = edge.Ahead;
        if (along < -edge.Behind) along = -edge.Behind;
        double px = edge.Point.X + edge.Direction.X * along, pz = edge.Point.Z + edge.Direction.Z * along;
        Target = new Vector3((float)(px + edge.HighSide.X * Inset), (float)edge.TopHeight, (float)(pz + edge.HighSide.Z * Inset));
        double lx = pos.x - px, lz = pos.z - pz;
        return (float)Math.Sqrt(lx * lx + lz * lz);
    }

    public static void Tick(float dt)
    {
        string error;
        if (cooldown > 0) SkateEdges.TryCooldown(cooldown, false, dt, out cooldown, out error);
    }

    public static void Release(float dt)
    {
        string error;
        if (!SkateEdges.TryCooldown(cooldown, true, dt, out cooldown, out error)) cooldown = SkateMotion.RailRecaptureDelay;
    }

    public static bool TryCapture(Vector3 pos, Vector3 velocity, float noseYaw, float dt)
    {
        if (!Enabled || failed || cooldown > 0) return false;
        try
        {
            double vx = velocity.x, vz = velocity.z;
            if (vx * vx + vz * vz < SkateMotion.MinimumCaptureSpeed * SkateMotion.MinimumCaptureSpeed) return false;
            var grid = Sample(pos, YawOf(vx, vz));
            SkateEdge found; string error;
            if (!SkateEdges.TryFind(grid, new SkateVector(vx, 0, vz), out found, out error)) { Last = "find refused: " + error; return false; }
            if (!found.Found) return false;
            Finds++;
            var previous = edge; edge = found;
            var off = Aim(pos);
            var feet = pos.y - (float)found.TopHeight;
            bool allowed = false; double speed = 0;
            var rad = noseYaw * Math.PI / 180.0;
            var ok = off <= Reach && feet >= -0.1f && feet <= 0.4f && found.Ahead >= 0.3
                && SkateEdges.TryCapture(found, new SkateVector(velocity.x, velocity.y, velocity.z), new SkateVector(Math.Sin(rad), 0, Math.Cos(rad)), cooldown, out allowed, out speed, out error) && allowed;
            if (!ok) { Last = "edge near: off=" + off.ToString("F2") + " feet=" + feet.ToString("F2") + " ahead=" + found.Ahead.ToString("F2") + " allowed=" + allowed; edge = previous; return false; }
            Speed = (float)speed; Heading = YawOf(found.Direction.X, found.Direction.Z); Captures++;
            Last = "captured top=" + found.TopHeight.ToString("F2") + " speed=" + Speed.ToString("F1") + " ahead=" + found.Ahead.ToString("F2") + " heading=" + Heading.ToString("F0");
            Out.Say("GRIND " + Last);
            return true;
        }
        catch (Exception e) { failed = true; Out.Say("GRIND scan threw " + e.GetType().Name + ": " + e.Message); return false; }
    }

    // One step along the edge: is it still there ahead, and is the board still fast enough.
    public static bool TryContinue(Vector3 pos, float dt, out bool ended)
    {
        ended = false;
        try
        {
            var grid = Sample(pos, Heading);
            bool continues; SkateEdge next; string error;
            if (!SkateEdges.TryContinue(edge, grid, edge.Direction, out continues, out next, out error)) { Last = "continue refused: " + error; return false; }
            if (!continues) { Last = "edge ended"; ended = true; return true; }
            if (next.Found) edge = next;
            double speed; bool grinding;
            if (!SkateEdges.TryGrindStep(Speed, dt, out speed, out grinding, out error)) { Last = "grind step refused: " + error; return false; }
            Speed = (float)speed;
            if (!grinding) { Last = "too slow"; ended = true; return true; }
            Heading = YawOf(edge.Direction.X, edge.Direction.Z);
            var off = Aim(pos);
            if (off > 0.8f || Math.Abs(pos.y - (float)edge.TopHeight) > 0.6f) { Last = "left the edge: off=" + off.ToString("F2"); ended = true; }
            return true;
        }
        catch (Exception e) { failed = true; Out.Say("GRIND step threw " + e.GetType().Name + ": " + e.Message); return false; }
    }
}
