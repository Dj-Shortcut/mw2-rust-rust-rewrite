// A controller reaches the game as keys, through Steam Input. Windows may show the same controller
// to this process directly as well (XInput), with sticks that are analogue where keys are not.
// Nothing rides on it yet: the log says whether one is shown and whether it answers, which is what
// has to be known before the sticks can be used. A slot without a controller is slow to ask, so
// the empty ones are asked only now and then.
using System;
using System.Runtime.InteropServices;

public static class SkatePad
{
    [StructLayout(LayoutKind.Sequential)]
    private struct State { public uint Packet; public ushort Buttons; public byte LeftTrigger, RightTrigger; public short LeftX, LeftY, RightX, RightY; }

    [DllImport("xinput1_4.dll", EntryPoint = "XInputGetState")]
    private static extern uint Read(uint slot, out State state);

    public static int Shown;
    public static bool Answers;
    private const short Far = 12000;
    private const byte Pulled = 60;
    private static bool failed, saidNone;
    private static float nextLook;
    private static int slots;

    public static void Poll(float now)
    {
        if (failed) return;
        try
        {
            State s;
            if (now >= nextLook)
            {
                nextLook = now + 3f;
                for (var i = 0; i < 4; i++)
                {
                    var bit = 1 << i;
                    if ((slots & bit) != 0 || Read((uint)i, out s) != 0) continue;
                    slots |= bit; Shown++;
                    Out.Say("INPUT a controller is shown to the game directly, in slot " + i);
                }
                if (slots == 0 && !saidNone) { saidNone = true; Out.Say("INPUT no controller is shown to the game directly"); }
            }
            for (var i = 0; i < 4; i++)
            {
                var bit = 1 << i;
                if ((slots & bit) == 0) continue;
                if (Read((uint)i, out s) != 0) { slots &= ~bit; Shown--; Out.Say("INPUT the controller in slot " + i + " is gone"); continue; }
                if (Answers) continue;
                if (s.Buttons == 0 && s.LeftTrigger < Pulled && s.RightTrigger < Pulled && Near(s.LeftX) && Near(s.LeftY) && Near(s.RightX) && Near(s.RightY)) continue;
                Answers = true;
                Out.Say("INPUT the controller in slot " + i + " answers directly: buttons=0x" + s.Buttons.ToString("X4") + " left stick=" + s.LeftX + "," + s.LeftY
                    + " right stick=" + s.RightX + "," + s.RightY + " triggers=" + s.LeftTrigger + "," + s.RightTrigger);
            }
        }
        catch (Exception e) { failed = true; Out.Say("INPUT a controller cannot be read directly: " + e.GetType().Name + ": " + e.Message); }
    }

    private static bool Near(short v) { return v < Far && v > -Far; }
}
