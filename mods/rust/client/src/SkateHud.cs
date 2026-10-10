// On-screen text for the ride, drawn with IMGUI (probe run M showed it works in this client).
// All text is English. Speed and score while riding, the name and points of each landed trick,
// the open combo, and the controls for a few seconds after getting on.
using System;
using UnityEngine;

public static class SkateHud
{
    public static float MountedAt = -99f;
    private static GUIStyle big, bigDark, mid, midDark, side, sideDark, small, smallDark;
    private static bool centred;

    private static GUIStyle Make(int size, Color colour)
    {
        var s = new GUIStyle(GUI.skin.label);
        s.fontSize = size; s.fontStyle = FontStyle.Bold;
        s.normal.textColor = colour;
        return s;
    }

    private static void Text(float x, float y, float width, string text, GUIStyle light, GUIStyle dark)
    {
        var height = light.fontSize * 1.6f;
        GUI.Label(new Rect(x + 2f, y + 2f, width, height), text, dark);
        GUI.Label(new Rect(x, y, width, height), text, light);
    }

    // Centred on the screen at height y. Falls back to an estimate of the text width when the
    // style's alignment cannot be set in this build.
    private static void Middle(float y, string text, GUIStyle light, GUIStyle dark)
    {
        float w = Screen.width;
        if (centred) Text(0f, y, w, text, light, dark);
        else { var guess = text.Length * light.fontSize * 0.56f; Text((w - guess) * 0.5f, y, guess + 40f, text, light, dark); }
    }

    public static void Draw()
    {
        var now = Time.realtimeSinceStartup;
        var popup = now - SkateRide.TrickAt < 2.2f;
        if (!Scenarios.Active && !SkateRide.On && !popup) return;
        float w = Screen.width, h = Screen.height;
        // Sizes are for a 1080-line screen and grow with it.
        var k = h / 1080f; if (k < 0.75f) k = 0.75f;
        if (big == null)
        {
            var dark = new Color(0f, 0f, 0f, 0.85f);
            int b = (int)(44 * k), m = (int)(27 * k), s = (int)(19 * k);
            big = Make(b, Color.white); bigDark = Make(b, dark); mid = Make(m, Color.white); midDark = Make(m, dark); side = Make(m, Color.white); sideDark = Make(m, dark); small = Make(s, Color.white); smallDark = Make(s, dark);
            try { big.alignment = bigDark.alignment = mid.alignment = midDark.alignment = TextAnchor.UpperCenter; centred = true; }
            catch (Exception e) { centred = false; Out.Say("HUD centred text unavailable: " + e.GetType().Name); }
        }
        if (Scenarios.Active) Text(40f * k, 110f * k, w - 80f * k, "SKATE TEST  " + Scenarios.Phase, small, smallDark);
        if (SkateRide.On)
        {
            Text(40f * k, h - 270f * k, 700f * k, (Math.Abs(SkateRide.Speed) * 3.6f).ToString("F0") + " km/h" + (SkateRide.Trick.Switch ? "   SWITCH" : ""), side, sideDark);
            Text(40f * k, h - 232f * k, 700f * k, "SCORE " + SkateRide.Trick.TotalPoints.ToString("N0"), side, sideDark);
            if (SkateRide.Trick.ComboCount > 0)
                Middle(h * 0.28f + 60f * k, "COMBO " + (SkateRide.Trick.ComboBasePoints * SkateRide.Trick.ComboCount).ToString("N0") + "   x" + SkateRide.Trick.ComboCount, mid, midDark);
            if (SkateRide.Mode == RideMode.Grind) Middle(h * 0.28f, "GRIND  " + SkateRide.Trick.GrindSeconds.ToString("F1") + " s", big, bigDark);
            if (now - MountedAt < 9f && !Scenarios.Active)
            {
                Text(40f * k, h - 186f * k, w - 80f * k, "W push   S brake   A / D carve   SPACE ollie   K or hold CTRL get off   V camera", small, smallDark);
                Text(40f * k, h - 158f * k, w - 80f * k, "In the air: tap W kickflip   tap S heelflip   A / D spin   hold CTRL grab", small, smallDark);
            }
        }
        if (popup && SkateRide.Mode != RideMode.Grind && SkateRide.TrickName != "")
            Middle(h * 0.28f, SkateRide.TrickName == "Bail" ? "BAIL" : SkateRide.TrickName + (SkateRide.TrickPoints > 0 ? "   +" + SkateRide.TrickPoints.ToString("N0") : ""), big, bigDark);
        if (now - SkateRide.BankedAt < 2.2f) Middle(h * 0.28f + 60f * k, "BANKED  +" + SkateRide.Banked.ToString("N0"), mid, midDark);
    }
}
