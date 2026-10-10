// Camera while riding. The game puts the main camera at the player's eyes every frame; this runs
// later in the frame (LateDriver) and can move it behind the rider. Probe run M showed that the
// moved main camera renders normally, so no second camera is needed for play. The view direction
// the player steers with is read before the camera is moved.
using System;
using UnityEngine;

public static class SkateCamera
{
    public static bool Chase;
    public static float LookYaw, LookPitch, Distance = 3.4f, Height = 1.7f;
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

    // Called last in the late pass.
    public static void Apply(Transform playerT)
    {
        if (!Chase || failed || playerT == null) return;
        try
        {
            var cam = Camera.main;
            if (cam == null) return;
            var pitch = SkateRide.Clamp(LookPitch, -20f, 60f);
            var back = Quaternion.Euler(pitch, LookYaw, 0f) * Vector3.forward;
            var focus = playerT.position + Vector3.up * 1.15f;
            var want = focus - back * Distance + Vector3.up * (Height - 1.15f) * 0.5f;
            // Keep the camera out of walls and the ground: pull it in along the line from the rider.
            var dir = want - focus; var len = dir.magnitude;
            RaycastHit hit;
            if (len > 0.01f && Physics.SphereCast(focus, 0.2f, dir / len, out hit, len, SkateRide.GroundMask, QueryTriggerInteraction.Ignore))
                want = focus + dir / len * Math.Max(0.6f, hit.m_Distance - 0.05f);
            cam.transform.position = want;
            cam.transform.LookAt(focus);
        }
        catch (Exception e) { failed = true; Out.Say("CAMERA chase threw " + e.GetType().Name + ": " + e.Message); }
    }
}
