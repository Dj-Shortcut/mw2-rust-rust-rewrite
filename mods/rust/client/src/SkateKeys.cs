// Input for the ride. Keys are read once per frame and presses are latched until a physics step
// has seen them, because the game steps physics at 32 Hz and draws frames more often. A scripted
// session sets the same fields itself. The legacy Input class is disabled in this client build, so
// this is the Input System keyboard; a controller arrives as keys through Steam Input.
//   W / S           push / brake on the ground; a fresh press in the air is a kickflip / heelflip
//   A / D           carve on the ground; spin in the air
//   Space           ollie
//   Left Ctrl or C  grab in the air; held on the ground it steps off the board
//   K               get on and off (two quick presses of Space also get on)
//   V               switch between the chase camera and first person
// Nothing is read while the cursor is free (console, chat, inventory, map), so typing never rides.
using System;
using UnityEngine;
using UnityEngine.InputSystem;

public static class SkateKeys
{
    public static bool Scripted;
    // Held.
    public static bool Push, Brake, Left, Right, Crouch, Jump;
    // Pressed since the last physics step.
    public static bool JumpPressed, FlipUp, FlipDown, LeftPressed, RightPressed;
    // Pressed this frame; consumed by the frame loop.
    public static bool JumpTap, ToggleTap, CameraTap, CrouchTap;
    public static float LookYaw;
    private static bool wasPush, wasBrake, wasLeft, wasRight, wasJump, wasCrouch, wasToggle, wasCamera;
    private static bool extraKeys = true, cursorKnown = true, failed;

    public static void Reset()
    {
        Push = Brake = Left = Right = Crouch = Jump = false;
        EndStep(); JumpTap = ToggleTap = CameraTap = CrouchTap = false;
    }

    // Every frame, before the frame loop looks at the taps.
    public static void Poll()
    {
        JumpTap = ToggleTap = CameraTap = CrouchTap = false;
        if (Scripted || failed) return;
        try
        {
            var kb = Keyboard.current;
            if (kb == null || CursorFree()) { Push = Brake = Left = Right = Crouch = Jump = false; wasPush = wasBrake = wasLeft = wasRight = wasJump = wasCrouch = wasToggle = wasCamera = false; return; }
            bool push = kb.wKey.isPressed, brake = kb.sKey.isPressed, left = kb.aKey.isPressed, right = kb.dKey.isPressed;
            bool jump = kb.spaceKey.isPressed, toggle = kb.kKey.isPressed, crouch = false, camera = false;
            if (extraKeys)
            {
                try { crouch = kb.leftCtrlKey.isPressed || kb.cKey.isPressed; camera = kb.vKey.isPressed; }
                catch (Exception e) { extraKeys = false; Out.Say("INPUT extra keys unavailable: " + e.GetType().Name + ": " + e.Message); }
            }
            if (push && !wasPush) FlipUp = true;
            if (brake && !wasBrake) FlipDown = true;
            if (left && !wasLeft) LeftPressed = true;
            if (right && !wasRight) RightPressed = true;
            if (jump && !wasJump) { JumpPressed = true; JumpTap = true; }
            if (crouch && !wasCrouch) CrouchTap = true;
            if (toggle && !wasToggle) ToggleTap = true;
            if (camera && !wasCamera) CameraTap = true;
            Push = push; Brake = brake; Left = left; Right = right; Jump = jump; Crouch = crouch;
            wasPush = push; wasBrake = brake; wasLeft = left; wasRight = right; wasJump = jump; wasCrouch = crouch; wasToggle = toggle; wasCamera = camera;
        }
        catch (Exception e) { failed = true; Reset(); Out.Say("INPUT key read threw " + e.GetType().Name + ": " + e.Message); }
    }

    // After a physics step has seen the latched presses.
    public static void EndStep() { JumpPressed = FlipUp = FlipDown = LeftPressed = RightPressed = false; }

    // The game locks the cursor while the player controls the character and frees it for every
    // window that takes text or clicks.
    private static bool CursorFree()
    {
        if (!cursorKnown) return false;
        try { return Cursor.lockState != CursorLockMode.Locked; }
        catch (Exception e) { cursorKnown = false; Out.Say("INPUT cursor state unavailable: " + e.GetType().Name + ": " + e.Message); return false; }
    }
}
