// What a ride with a controller leaves behind to be read afterwards: each movement of the right
// stick with what came of it, in the notation that the offline flick test replays; the keys and
// mouse buttons that Steam's layout sends for the controller's buttons; and whether the right stick
// also turns the game's own view. Every kind of line is bounded for a session.
using System;
using System.Collections.Generic;
using System.Text;
using Shortcut.RustMod;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;

public static class SkatePadLog
{
    private const float Begin = 0.3f, Rest = 0.2f, Far = 0.6f, Pair = 0.2f;
    private const long StillUs = 90000;
    private const int MostPaths = 300, MostPoints = 60, MostPairs = 60, MostViews = 4;
    private static readonly string[] Names = { "D-pad up", "D-pad down", "D-pad left", "D-pad right", "Start", "Back", "left stick click", "right stick click", "LB", "RB", "", "", "A", "B", "X", "Y" };

    private static void Say(string m) { Out.Say("PAD " + m); }

    // --- the right stick, movement by movement
    private static readonly StringBuilder text = new StringBuilder();
    private static bool open, away, manual, moved, pushed;
    private static long startUs, lastUs, restUs;
    private static int points, paths;
    private static SkateFlickEvents got;
    private static float pop;
    private static string during = "";

    public static void Stick(long us, float x, float y)
    {
        var far = x * x + y * y;
        pushed = far > Begin * Begin;
        if (pushed) moved = true;
        if (!open)
        {
            if (far < Begin * Begin || paths >= MostPaths) return;
            open = true; startUs = us; text.Length = 0; points = 0; got = SkateFlickEvents.None; pop = 0f; manual = false;
            during = SkateRide.On ? SkateRide.Mode.ToString() : "on foot";
        }
        lastUs = us;
        if (far > Rest * Rest) away = true; else if (away) { away = false; restUs = us; }
        if (points < MostPoints) { points++; text.Append(' ').Append((us - startUs) / 1000).Append(':').Append((int)Math.Round(x * 100f)).Append(',').Append((int)Math.Round(y * 100f)); }
    }

    public static void Fired(SkateFlickEvents events, float strength) { if (open) { got |= events; pop = strength; } }

    // By the reader's clock: a movement is over when the stick has been back in the middle for a moment.
    public static void Clock(long us)
    {
        if (!open) return;
        if (SkatePad.Manual) manual = true;
        if (away || us - restUs < StillUs) return;
        open = false; paths++;
        var came = got == SkateFlickEvents.None ? (manual ? "manual" : "nothing") : got + (manual ? ", manual" : "") + (pop > 0f ? " pop=" + pop.ToString("F2") : "");
        Say("stick " + ((lastUs - startUs) / 1000) + " ms, " + during + " -> " + came + " |" + text);
    }

    // --- what goes down on the controller, to pair with the keys that follow
    private static readonly List<string> recentName = new List<string>();
    private static readonly List<float> recentAt = new List<float>();
    private static readonly List<string> pairs = new List<string>();
    private static int held, leftWay, rightWay;
    private static bool leftPulled, rightPulled, known;

    public static void Buttons(int buttons, float lt, float rt, float lx, float ly, float rx, float ry, float now, bool edges)
    {
        bool l = lt > SkatePad.Pulled, r = rt > SkatePad.Pulled;
        int lw = Way(lx, ly), rw = Way(rx, ry);
        if (edges && known)
        {
            for (var i = 0; i < 16; i++)
                if ((buttons & ~held & (1 << i)) != 0 && Names[i] != "") Edge(Names[i], now);
            if (l && !leftPulled) Edge("LT", now);
            if (r && !rightPulled) Edge("RT", now);
            if (lw != leftWay && lw != 0) Edge("left stick " + WayName(lw), now);
            if (rw != rightWay && rw != 0) Edge("right stick " + WayName(rw), now);
        }
        held = buttons; leftPulled = l; rightPulled = r; leftWay = lw; rightWay = rw; known = true;
    }

    // A stick pushed far one way counts as a button: Steam's layout sends a key for that too.
    private static int Way(float x, float y) { return y > Far ? 1 : y < -Far ? 2 : x < -Far ? 3 : x > Far ? 4 : 0; }
    private static string WayName(int way) { return way == 1 ? "up" : way == 2 ? "down" : way == 3 ? "left" : "right"; }

    private static void Edge(string name, float now)
    {
        while (recentAt.Count > 0 && now - recentAt[0] > 1f) { recentAt.RemoveAt(0); recentName.RemoveAt(0); }
        recentName.Add(name); recentAt.Add(now);
    }

    // --- the keys and mouse buttons that arrive, and the view
    private static IntPtr board;
    private static ButtonControl[] keys;
    private static string[] keyNames;
    private static bool[] keyWas;
    private static bool keysFailed, keysPrimed, lookKnown;
    private static float lookYaw, lookPitch, turned, nextView;
    private static int viewsOnFoot, viewsRiding;

    public static void Frame(float now)
    {
        if (!SkatePad.Shared) { keysPrimed = lookKnown = pushed = false; return; }
        Keys(now);
        View(now);
    }

    private static void Keys(float now)
    {
        if (keysFailed) return;
        try
        {
            var kb = Keyboard.current;
            if (kb == null) return;
            // Asking twice can hand out two wrappers for one keyboard, so the keyboard is told by its address.
            if (keys == null || kb.Pointer != board)
            {
                board = kb.Pointer; keysPrimed = false;
                var k = new List<ButtonControl>(); var n = new List<string>();
                Add(k, n, "A", kb.aKey); Add(k, n, "B", kb.bKey); Add(k, n, "C", kb.cKey); Add(k, n, "D", kb.dKey); Add(k, n, "E", kb.eKey); Add(k, n, "F", kb.fKey); Add(k, n, "G", kb.gKey);
                Add(k, n, "H", kb.hKey); Add(k, n, "I", kb.iKey); Add(k, n, "J", kb.jKey); Add(k, n, "K", kb.kKey); Add(k, n, "L", kb.lKey); Add(k, n, "M", kb.mKey); Add(k, n, "N", kb.nKey);
                Add(k, n, "O", kb.oKey); Add(k, n, "P", kb.pKey); Add(k, n, "Q", kb.qKey); Add(k, n, "R", kb.rKey); Add(k, n, "S", kb.sKey); Add(k, n, "T", kb.tKey); Add(k, n, "U", kb.uKey);
                Add(k, n, "V", kb.vKey); Add(k, n, "W", kb.wKey); Add(k, n, "X", kb.xKey); Add(k, n, "Y", kb.yKey); Add(k, n, "Z", kb.zKey);
                Add(k, n, "0", kb.digit0Key); Add(k, n, "1", kb.digit1Key); Add(k, n, "2", kb.digit2Key); Add(k, n, "3", kb.digit3Key); Add(k, n, "4", kb.digit4Key);
                Add(k, n, "5", kb.digit5Key); Add(k, n, "6", kb.digit6Key); Add(k, n, "7", kb.digit7Key); Add(k, n, "8", kb.digit8Key); Add(k, n, "9", kb.digit9Key);
                Add(k, n, "Space", kb.spaceKey); Add(k, n, "Enter", kb.enterKey); Add(k, n, "Tab", kb.tabKey); Add(k, n, "Escape", kb.escapeKey); Add(k, n, "Backspace", kb.backspaceKey);
                Add(k, n, "LeftShift", kb.leftShiftKey); Add(k, n, "RightShift", kb.rightShiftKey); Add(k, n, "LeftCtrl", kb.leftCtrlKey); Add(k, n, "RightCtrl", kb.rightCtrlKey);
                Add(k, n, "LeftAlt", kb.leftAltKey); Add(k, n, "RightAlt", kb.rightAltKey); Add(k, n, "CapsLock", kb.capsLockKey);
                Add(k, n, "ArrowUp", kb.upArrowKey); Add(k, n, "ArrowDown", kb.downArrowKey); Add(k, n, "ArrowLeft", kb.leftArrowKey); Add(k, n, "ArrowRight", kb.rightArrowKey);
                Add(k, n, "Backquote", kb.backquoteKey); Add(k, n, "F1", kb.f1Key); Add(k, n, "F2", kb.f2Key); Add(k, n, "F3", kb.f3Key); Add(k, n, "F4", kb.f4Key); Add(k, n, "F5", kb.f5Key);
                var mouse = Mouse.current;
                if (mouse != null)
                {
                    Add(k, n, "mouse left", mouse.leftButton); Add(k, n, "mouse right", mouse.rightButton); Add(k, n, "mouse middle", mouse.middleButton);
                    Add(k, n, "mouse forward", mouse.forwardButton); Add(k, n, "mouse back", mouse.backButton);
                }
                keys = k.ToArray(); keyNames = n.ToArray(); keyWas = new bool[keys.Length];
            }
            // A key that was already down when the watching began is nobody's press.
            if (!keysPrimed)
            {
                for (var i = 0; i < keys.Length; i++) keyWas[i] = keys[i].isPressed;
                keysPrimed = true;
                return;
            }
            for (var i = 0; i < keys.Length; i++)
            {
                var down = keys[i].isPressed; var tapped = !down && !keyWas[i] && keys[i].wasPressedThisFrame;
                if ((down && !keyWas[i]) || tapped) Key(keyNames[i], now);
                keyWas[i] = down;
            }
        }
        catch (Exception e) { keysFailed = true; Say("the keys beside the controller cannot be watched: " + e.GetType().Name + ": " + e.Message); }
    }

    private static void Add(List<ButtonControl> k, List<string> n, string name, ButtonControl key) { if (key != null) { k.Add(key); n.Add(name); } }

    // A key that goes down right after something on the controller is what the layout sends for it.
    private static void Key(string key, float now)
    {
        var from = "";
        for (var i = recentName.Count - 1; i >= 0; i--)
        {
            if (now - recentAt[i] > Pair) break;
            from += (from == "" ? "" : " or ") + recentName[i];
        }
        if (from == "") return;
        var pair = from + " -> " + key;
        if (pairs.Contains(pair) || pairs.Count >= MostPairs) return;
        pairs.Add(pair);
        Say("layout: " + pair);
    }

    // The game's own view as sampled before the chase camera moves the camera. It is only sampled
    // once the player is up: a second without a sample says nothing about the view.
    private static void View(float now)
    {
        if (now - SkateCamera.SampledAt > 0.25f) { lookKnown = moved = false; turned = 0f; nextView = now + 1f; return; }
        float yaw = SkateCamera.LookYaw, pitch = SkateCamera.LookPitch;
        if (lookKnown) turned += Math.Abs(SkateRide.Delta(lookYaw, yaw)) + Math.Abs(SkateRide.Delta(lookPitch, pitch));
        lookYaw = yaw; lookPitch = pitch; lookKnown = true;
        if (pushed) moved = true;
        if (now < nextView) return;
        nextView = now + 1f;
        if (moved && (SkateRide.On ? viewsRiding : viewsOnFoot) < MostViews)
        {
            if (SkateRide.On) viewsRiding++; else viewsOnFoot++;
            Say("the game's view turned " + turned.ToString("F0") + " degrees in a second in which the right stick was moved, " + (SkateRide.On ? "on the board" : "on foot"));
        }
        turned = 0f; moved = false;
    }
}
