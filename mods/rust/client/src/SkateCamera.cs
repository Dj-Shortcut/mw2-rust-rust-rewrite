// Camera while riding. The game puts the main camera at the player's eyes every frame; this runs
// later in the frame and moves it behind the rider. The moved main camera renders normally, so no
// second camera is needed (one made with CopyFrom lacks the main camera's image effects and draws
// the rider white). The view direction the player steers with is read before the camera is moved.
// Test sessions can instead fix the camera at a bearing around the board.
using System;
using UnityEngine;

public static class SkateCamera
{
    public static bool Chase = true, Fixed;
    public static float LookYaw, LookPitch, Distance = 3.4f, Bearing = 180f, FixedDistance = 2.7f, FixedHeight = 1.15f;
    private static bool failed;

    // Called first in the late pass: remember where the game's own camera looks.
    public static void Sample()
    {
        var cam = Camera.main;
        if (cam == null) return;
        var e = cam.transform.eulerAngles;
        LookYaw = e.y;
        LookPitch = e.x > 180f ? e.x - 360f : e.x;
    }

    // Called last in the late pass. `at` is where the board touches the ground under the rider.
    public static void Apply(Vector3 at)
    {
        if (failed || !(Chase || Fixed)) return;
        try
        {
            var cam = Camera.main;
            if (cam == null) return;
            var focus = at + Vector3.up * 1.05f;
            Vector3 want;
            if (Fixed)
            {
                // Bearing from the rider to the camera, measured from the direction of travel:
                // 0 ahead of the board, 90 on its right, 180 behind it.
                want = focus + SkateRide.Dir(SkateRide.Yaw + Bearing) * FixedDistance + Vector3.up * (FixedHeight - 1.05f);
            }
            else
            {
                var pitch = SkateRide.Clamp(LookPitch, -20f, 60f);
                var back = Quaternion.Euler(pitch, LookYaw, 0f) * Vector3.forward;
                want = focus - back * Distance + Vector3.up * 0.3f;
                // Keep the camera out of walls and the ground: pull it in along the line from the rider.
                var dir = want - focus; var len = dir.magnitude;
                RaycastHit hit;
                if (len > 0.01f && Physics.SphereCast(focus, 0.2f, dir / len, out hit, len, SkateRide.GroundMask, QueryTriggerInteraction.Ignore))
                    want = focus + dir / len * Math.Max(0.6f, hit.m_Distance - 0.05f);
            }
            cam.transform.position = want;
            cam.transform.LookAt(focus);
        }
        catch (Exception e) { failed = true; Out.Say("CAMERA threw " + e.GetType().Name + ": " + e.Message); }
    }
}
