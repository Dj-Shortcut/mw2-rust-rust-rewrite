// The legacy Input class is disabled in this client build; a controller arrives as keys through
// Steam Input, and its sticks only through the reader outside the game (SkatePad). Presses are
// latched until a physics step has seen them: the game steps physics at 32 Hz and draws frames
// more often. Nothing is read while the cursor is free (console, chat, inventory), so typing never
// rides.
// With a controller the layout is that of the skate. games: left stick steers, right stick is the
// board (SkateFlick), A and X push, B brakes, Y gets on and off, a trigger grabs. Steam's layout
// keeps sending its keys for the same buttons, so while the controller rides the keys do not.
using System;
using Shortcut.RustMod;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;

public static class SkateKeys
{
    public const float PadRest = 0.5f, SteerDead = 0.18f, Straight = 0.35f, StickPush = 0.75f, SpinFrom = 0.5f, LeastPop = 0.7f;
    public static bool Scripted;
    public static bool Push, Brake, Left, Right, Crouch, Jump, Manual;
    public static bool JumpPressed, FlipUp, FlipDown, LeftPressed, RightPressed;
    public static bool JumpTap, ToggleTap, CameraTap, CrouchTap;
    public static float LookYaw;
    // Pad: the controller rides, not the keyboard. Analog: Steer is a rate, not a direction to turn to.
    public static bool Pad, Analog, Charging;
    public static float Steer, Charge, PopScale = 1f;
    private static bool wasPush, wasBrake, wasLeft, wasRight, wasJump, wasCrouch, wasToggle, wasCamera, wasPadLeft, wasPadRight;
    private static bool crouchKeys = true, cameraKey = true, manualKey = true, taps = true, cursorKnown = true, failed;

    public static void Reset()
    {
        Push = Brake = Left = Right = Crouch = Jump = Manual = Analog = Charging = false; Steer = Charge = 0f; PopScale = 1f;
        EndStep(); JumpTap = ToggleTap = CameraTap = CrouchTap = false;
    }

    public static void Poll()
    {
        JumpTap = ToggleTap = CameraTap = CrouchTap = false;
        if (Scripted || failed) return;
        var now = Time.realtimeSinceStartup;
        SkatePad.Poll(now);
        var padDown = SkatePad.Pressed();
        SkatePadLog.Frame(now);
        try
        {
            var kb = Keyboard.current;
            if (kb == null || CursorFree())
            {
                Push = Brake = Left = Right = Crouch = Jump = Manual = Charging = false; Steer = Charge = 0f;
                wasPush = wasBrake = wasLeft = wasRight = wasJump = wasCrouch = wasToggle = wasCamera = wasPadLeft = wasPadRight = false; EndStep(); SkatePad.TakeFlick();
                return;
            }
            bool push = Down(kb.wKey), brake = Down(kb.sKey), left = Down(kb.aKey), right = Down(kb.dKey);
            bool jump = Down(kb.spaceKey), toggle = Down(kb.kKey), crouch = false, camera = false, manual = false;
            if (crouchKeys)
            {
                try { crouch = Down(kb.leftCtrlKey) || Down(kb.cKey); }
                catch (Exception e) { crouchKeys = false; Out.Say("INPUT crouch keys unavailable: " + e.GetType().Name + ": " + e.Message); }
            }
            if (cameraKey)
            {
                try { camera = Down(kb.lKey); }
                catch (Exception e) { cameraKey = false; Out.Say("INPUT camera key unavailable: " + e.GetType().Name + ": " + e.Message); }
            }
            if (manualKey)
            {
                try { manual = kb.leftShiftKey.isPressed; }
                catch (Exception e) { manualKey = false; Out.Say("INPUT manual key unavailable: " + e.GetType().Name + ": " + e.Message); }
            }
            // Whose hands are on the ride: the controller's from the moment it is touched; the
            // keyboard's again when a riding key goes down while the controller has been at rest,
            // which Steam's layout cannot have sent.
            var keyDown = (push && !wasPush) || (brake && !wasBrake) || (left && !wasLeft) || (right && !wasRight) || (jump && !wasJump) || (crouch && !wasCrouch);
            var had = Pad;
            if (!SkatePad.Shared) Pad = false;
            else if (now - SkatePad.TouchedAt < 0.05f) Pad = true;
            else if (keyDown && now - SkatePad.TouchedAt > PadRest) Pad = false;
            if (Pad != had) Out.Say("INPUT the " + (Pad ? "controller" : "keyboard") + " rides");

            if (toggle && !wasToggle) ToggleTap = true;
            if (camera && !wasCamera) CameraTap = true;
            // Two quick jumps get on the board, whoever rides: Steam sends the jump key for the controller too.
            if (jump && !wasJump) JumpTap = true;
            if (Pad)
            {
                if ((padDown & SkatePad.Y) != 0) ToggleTap = true;
                if ((padDown & SkatePad.RightStickClick) != 0) CameraTap = true;
                float x = SkatePad.LeftX, y = SkatePad.LeftY;
                var straight = Math.Abs(x) < Straight;
                Analog = true;
                Steer = Math.Abs(x) <= SteerDead ? 0f : (x - Math.Sign(x) * SteerDead) / (1f - SteerDead);
                Push = (SkatePad.Buttons & (SkatePad.A | SkatePad.X)) != 0 || (straight && y > StickPush);
                Brake = (SkatePad.Buttons & SkatePad.B) != 0 || (straight && y < -StickPush);
                Left = x < -SpinFrom; Right = x > SpinFrom;
                if (Left && !wasPadLeft) LeftPressed = true;
                if (Right && !wasPadRight) RightPressed = true;
                wasPadLeft = Left; wasPadRight = Right;
                Crouch = SkatePad.LeftTrigger > SkatePad.Pulled || SkatePad.RightTrigger > SkatePad.Pulled;
                Manual = SkatePad.Manual; Jump = false;
                Charging = SkatePad.Charging; Charge = SkatePad.Charge;
                var flick = SkatePad.TakeFlick();
                if ((flick & (SkateFlickEvents.Ollie | SkateFlickEvents.Nollie)) != 0) { JumpPressed = true; PopScale = LeastPop + (1f - LeastPop) * SkatePad.Pop; }
                if ((flick & SkateFlickEvents.Kickflip) != 0) FlipUp = true;
                if ((flick & SkateFlickEvents.Heelflip) != 0) FlipDown = true;
            }
            else
            {
                SkatePad.TakeFlick();
                Analog = Charging = false; Charge = 0f; wasPadLeft = wasPadRight = false;
                Steer = (left ? -1f : 0f) + (right ? 1f : 0f);
                if (push && !wasPush) FlipUp = true;
                if (brake && !wasBrake) FlipDown = true;
                if (left && !wasLeft) LeftPressed = true;
                if (right && !wasRight) RightPressed = true;
                if (jump && !wasJump) { JumpPressed = true; PopScale = 1f; }
                if (crouch && !wasCrouch) CrouchTap = true;
                Push = push; Brake = brake; Left = left; Right = right; Jump = jump; Crouch = crouch; Manual = manual;
            }
            wasPush = push; wasBrake = brake; wasLeft = left; wasRight = right; wasJump = jump; wasCrouch = crouch; wasToggle = toggle; wasCamera = camera;
        }
        catch (Exception e) { failed = true; Reset(); Out.Say("INPUT key read threw " + e.GetType().Name + ": " + e.Message); }
    }

    public static void EndStep() { JumpPressed = FlipUp = FlipDown = LeftPressed = RightPressed = false; }

    // A tap can begin and end between two frames; the key's state alone never shows such a tap.
    public static bool Down(KeyControl key)
    {
        if (key.isPressed) return true;
        if (!taps) return false;
        try { return key.wasPressedThisFrame; }
        catch (Exception e) { taps = false; Out.Say("INPUT taps between frames cannot be seen: " + e.GetType().Name + ": " + e.Message); return false; }
    }

    private static bool CursorFree()
    {
        if (!cursorKnown) return false;
        try { return Cursor.lockState != CursorLockMode.Locked; }
        catch (Exception e) { cursorKnown = false; Out.Say("INPUT cursor state unavailable: " + e.GetType().Name + ": " + e.Message); return false; }
    }
}
