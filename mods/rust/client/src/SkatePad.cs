// Test build, not for main: finds out how the controller can be read and what Steam's layout sends
// to the game for it. Inside the game Steam Input stands between a controller and whoever asks for
// it, so three views are logged side by side, each with the moment it was seen:
//   - the reader outside the game (tools/sticks.ps1), through its named block of memory;
//   - XInput asked from inside the game;
//   - the keys and mouse buttons that arrive, and how far the view turned.
// A key that goes down right after a button of the controller is logged as what that button sends.
using System;
using System.Collections.Generic;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;

public static class SkatePad
{
    public const string MapName = "ShortcutSkatePad";
    public const uint Magic = 0x44504B53;
    public const int Ring = 128, SampleBytes = 24, Header = 32, Size = Header + Ring * SampleBytes;
    private const float Far = 0.6f, Begin = 0.35f, Rest = 0.2f, Pair = 0.2f;
    private static readonly string[] Names = { "up", "down", "left", "right", "start", "back", "left stick click", "right stick click", "LB", "RB", "", "", "A", "B", "X", "Y" };

    [StructLayout(LayoutKind.Sequential)]
    private struct State { public uint Packet; public ushort Buttons; public byte LeftTrigger, RightTrigger; public short LeftX, LeftY, RightX, RightY; }

    [DllImport("xinput1_4.dll", EntryPoint = "XInputGetState")]
    private static extern uint Read(uint slot, out State state);

    private static void Say(string m) { Out.Say("PAD " + m); }
    private static int lines;
    private static void Few(string m) { if (lines < 600) { lines++; Out.Say(m); } }

    // --- the reader outside the game
    private static MemoryMappedFile map; private static MemoryMappedViewAccessor view;
    private static readonly byte[] block = new byte[Size];
    private static uint count; private static bool primed, sharedFailed, saidNone; private static float nextOpen; private static int sharedSlot = -2;
    private static int held; private static bool leftPulled, rightPulled; private static int leftWay, rightWay;
    private static readonly List<string> recentName = new List<string>(); private static readonly List<float> recentAt = new List<float>();
    private static readonly List<string> pairs = new List<string>();
    private static bool rightMoved;

    private sealed class Path
    {
        public readonly string Name; public bool Open; public long StartUs, LastUs, StillUs; public int Lines; public readonly StringBuilder Text = new StringBuilder(); public int Points;
        public Path(string name) { Name = name; }
    }
    private static readonly Path leftPath = new Path("left stick"), rightPath = new Path("right stick");

    public static void Poll(float now)
    {
        Shared(now);
        Inside(now);
        Keys(now);
        Look(now);
    }

    private static void Shared(float now)
    {
        if (sharedFailed) return;
        try
        {
            if (view == null)
            {
                if (now < nextOpen) return;
                nextOpen = now + 2f;
                try { map = MemoryMappedFile.OpenExisting(MapName, MemoryMappedFileRights.Read); }
                catch (System.IO.FileNotFoundException) { if (!saidNone) { saidNone = true; Say("no reader outside the game yet"); } return; }
                view = map.CreateViewAccessor(0, Size, MemoryMappedFileAccess.Read);
                Say("the reader outside the game is there");
            }
            view.ReadArray(0, block, 0, Size);
            if (BitConverter.ToUInt32(block, 0) != Magic)
            {
                Say("the reader outside the game has stopped");
                view.Dispose(); map.Dispose(); view = null; map = null; primed = false; sharedSlot = -2;
                return;
            }
            var slot = (int)BitConverter.ToUInt32(block, 12);
            if (slot != sharedSlot) { sharedSlot = slot; Say("reader: " + (slot < 0 ? "no controller" : "controller in slot " + slot) + ", slots shown mask=" + BitConverter.ToUInt32(block, 24) + " t=" + now.ToString("F2")); }
            var n = BitConverter.ToUInt32(block, 8);
            if (!primed) { primed = true; count = n; }
            if (n - count > Ring) { Say("missed " + (n - count - Ring) + " state changes"); count = n - Ring; }
            for (; count != n; count++) Sample(Header + (int)(count % Ring) * SampleBytes, now);
            var beat = BitConverter.ToInt64(block, 16);
            Close(leftPath, beat); Close(rightPath, beat);
        }
        catch (Exception e) { sharedFailed = true; Say("the shared controller cannot be read: " + e.GetType().Name + ": " + e.Message); }
    }

    private static void Sample(int at, float now)
    {
        var us = BitConverter.ToInt64(block, at);
        int buttons = BitConverter.ToUInt16(block, at + 12);
        var lt = block[at + 14] > 100; var rt = block[at + 15] > 100;
        float lx = BitConverter.ToInt16(block, at + 16) / 32768f, ly = BitConverter.ToInt16(block, at + 18) / 32768f;
        float rx = BitConverter.ToInt16(block, at + 20) / 32768f, ry = BitConverter.ToInt16(block, at + 22) / 32768f;
        for (var i = 0; i < 16; i++)
        {
            var bit = 1 << i;
            if (((buttons ^ held) & bit) != 0 && Names[i] != "") Edge(Names[i], (buttons & bit) != 0, now);
        }
        held = buttons;
        if (lt != leftPulled) { leftPulled = lt; Edge("LT", lt, now); }
        if (rt != rightPulled) { rightPulled = rt; Edge("RT", rt, now); }
        leftWay = Way("left stick", lx, ly, leftWay, now);
        rightWay = Way("right stick", rx, ry, rightWay, now);
        if (rx * rx + ry * ry > Begin * Begin) rightMoved = true;
        Trace(leftPath, us, lx, ly); Trace(rightPath, us, rx, ry);
    }

    // A stick pushed far one way counts as a button for pairing with keys: 1 up, 2 down, 3 left, 4 right.
    private static int Way(string stick, float x, float y, int was, float now)
    {
        var way = y > Far ? 1 : y < -Far ? 2 : x < -Far ? 3 : x > Far ? 4 : 0;
        if (way != was && way != 0) Edge(stick + (way == 1 ? " up" : way == 2 ? " down" : way == 3 ? " left" : " right"), true, now);
        return way;
    }

    private static void Edge(string name, bool down, float now)
    {
        if (!down) return;
        Few("PAD " + name + " t=" + now.ToString("F2"));
        recentName.Add(name); recentAt.Add(now);
        while (recentAt.Count > 0 && now - recentAt[0] > 1f) { recentAt.RemoveAt(0); recentName.RemoveAt(0); }
    }

    private static void Trace(Path p, long us, float x, float y)
    {
        var far = x * x + y * y;
        if (!p.Open)
        {
            if (far < Begin * Begin || p.Lines >= (p == rightPath ? 60 : 15)) return;
            p.Open = true; p.StartUs = us; p.Text.Length = 0; p.Points = 0;
        }
        p.LastUs = us;
        if (far > Rest * Rest) p.StillUs = us;
        if (p.Points < 70) { p.Points++; p.Text.Append(' ').Append((us - p.StartUs) / 1000).Append(':').Append((int)(x * 100)).Append(',').Append((int)(y * 100)); }
    }

    // A path ends when the stick has been back near the middle for a moment, by the reader's clock.
    private static void Close(Path p, long beat)
    {
        if (!p.Open || beat - p.StillUs < 90000) return;
        p.Open = false; p.Lines++;
        Out.Say("PATH " + p.Name + " " + ((p.LastUs - p.StartUs) / 1000) + " ms, time:x,y in percent:" + p.Text);
    }

    // --- XInput asked from inside the game
    private static int slots, insideHeld; private static bool insideFailed, insideAnswers, insideNone; private static float nextSlots;

    private static void Inside(float now)
    {
        if (insideFailed) return;
        try
        {
            State s;
            if (now >= nextSlots)
            {
                nextSlots = now + 3f;
                for (var i = 0; i < 4; i++)
                {
                    var bit = 1 << i;
                    if ((slots & bit) != 0 || Read((uint)i, out s) != 0) continue;
                    slots |= bit;
                    Out.Say("INSIDE a controller is shown to the game itself, in slot " + i + " t=" + now.ToString("F2"));
                }
                if (slots == 0 && !insideNone) { insideNone = true; Out.Say("INSIDE no controller is shown to the game itself"); }
            }
            for (var i = 0; i < 4; i++)
            {
                var bit = 1 << i;
                if ((slots & bit) == 0) continue;
                if (Read((uint)i, out s) != 0) { slots &= ~bit; Out.Say("INSIDE the controller in slot " + i + " is gone"); continue; }
                var moved = s.Buttons != 0 || s.LeftTrigger > 100 || s.RightTrigger > 100 || Math.Abs((int)s.LeftX) > 19000 || Math.Abs((int)s.LeftY) > 19000 || Math.Abs((int)s.RightX) > 19000 || Math.Abs((int)s.RightY) > 19000;
                if (moved && !insideAnswers)
                {
                    insideAnswers = true;
                    Out.Say("INSIDE the controller in slot " + i + " answers: buttons=0x" + s.Buttons.ToString("X4") + " left stick=" + s.LeftX + "," + s.LeftY + " right stick=" + s.RightX + "," + s.RightY
                        + " triggers=" + s.LeftTrigger + "," + s.RightTrigger + " t=" + now.ToString("F2"));
                }
                if (s.Buttons != insideHeld)
                {
                    for (var b = 0; b < 16; b++)
                        if (((s.Buttons ^ insideHeld) & (1 << b) & s.Buttons) != 0 && Names[b] != "") Few("INSIDE " + Names[b] + " t=" + now.ToString("F2"));
                    insideHeld = s.Buttons;
                }
            }
        }
        catch (Exception e) { insideFailed = true; Out.Say("INSIDE a controller cannot be asked for from inside the game: " + e.GetType().Name + ": " + e.Message); }
    }

    // --- what arrives as keys and mouse buttons
    private static IntPtr board; private static ButtonControl[] keys; private static string[] keyNames; private static bool[] keyWas; private static bool keysFailed;
    private static string pad = "?";

    private static void Keys(float now)
    {
        if (keysFailed) return;
        try
        {
            var kb = Keyboard.current; var mouse = Mouse.current;
            if (kb == null) return;
            // Asking twice can hand out two wrappers for one keyboard, so the keyboard is told by its address.
            if (kb.Pointer != board || keys == null)
            {
                board = kb.Pointer;
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
                if (mouse != null)
                {
                    Add(k, n, "mouse left", mouse.leftButton); Add(k, n, "mouse right", mouse.rightButton); Add(k, n, "mouse middle", mouse.middleButton);
                    Add(k, n, "mouse forward", mouse.forwardButton); Add(k, n, "mouse back", mouse.backButton);
                }
                keys = k.ToArray(); keyNames = n.ToArray(); keyWas = new bool[keys.Length];
                Out.Say("KEYS watching " + keys.Length + " keys and mouse buttons");
            }
            for (var i = 0; i < keys.Length; i++)
            {
                var down = keys[i].isPressed; var tapped = !down && !keyWas[i] && keys[i].wasPressedThisFrame;
                if ((down && !keyWas[i]) || tapped) Key(keyNames[i], now);
                keyWas[i] = down;
            }
            var seen = "none";
            try { var g = Gamepad.current; if (g != null) seen = g.name; } catch (Exception e) { seen = "cannot be asked: " + e.GetType().Name; }
            if (seen != pad) { pad = seen; Out.Say("UNITY the engine's own gamepad: " + seen + " t=" + now.ToString("F2")); }
        }
        catch (Exception e) { keysFailed = true; Out.Say("KEYS cannot be watched: " + e.GetType().Name + ": " + e.Message); }
    }

    private static void Add(List<ButtonControl> k, List<string> n, string name, ButtonControl key) { if (key != null) { k.Add(key); n.Add(name); } }

    private static void Key(string key, float now)
    {
        var from = "";
        for (var i = recentName.Count - 1; i >= 0; i--)
        {
            if (now - recentAt[i] > Pair) break;
            from += (from == "" ? "" : " or ") + recentName[i];
        }
        if (from == "") { Few("KEY " + key + " t=" + now.ToString("F2") + " (no controller button just before)"); return; }
        Few("KEY " + key + " t=" + now.ToString("F2") + " after " + from);
        var pair = from + " -> " + key;
        if (pairs.Contains(pair)) return;
        pairs.Add(pair);
        Out.Say("LAYOUT " + pair);
    }

    // --- how far the view turned, second by second, and whether the right stick was moved meanwhile
    private static float lookYaw, lookPitch, turned, nextLook; private static bool lookKnown; private static int lookLines;

    private static void Look(float now)
    {
        // The game's own view as sampled before the chase camera moves the camera.
        var yaw = SkateCamera.LookYaw; var pitch = SkateCamera.LookPitch;
        if (lookKnown) turned += Math.Abs(SkateRide.Delta(lookYaw, yaw)) + Math.Abs(SkateRide.Delta(lookPitch, pitch));
        lookYaw = yaw; lookPitch = pitch; lookKnown = true;
        if (now < nextLook) return;
        nextLook = now + 1f;
        if ((turned > 2f || rightMoved) && lookLines < 60) { lookLines++; Out.Say("VIEW turned " + turned.ToString("F0") + " degrees in a second, right stick " + (rightMoved ? "moved" : "still") + " t=" + now.ToString("F2")); }
        turned = 0f; rightMoved = false;
    }
}
