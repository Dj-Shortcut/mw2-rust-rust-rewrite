// The legacy Input class is disabled in this client build; a controller arrives as keys through
// Steam Input. Presses are latched until a physics step has seen them: the game steps physics at
// 32 Hz and draws frames more often. Nothing is read while the cursor is free (console, chat,
// inventory), so typing never rides.
using System;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.Controls;

public static class SkateKeys
{
    public static bool Scripted;
    public static bool Push, Brake, Left, Right, Crouch, Jump, Manual;
    public static bool JumpPressed, FlipUp, FlipDown, LeftPressed, RightPressed;
    public static bool JumpTap, ToggleTap, CameraTap, CrouchTap;
    public static float LookYaw;
    private static bool wasPush, wasBrake, wasLeft, wasRight, wasJump, wasCrouch, wasToggle, wasCamera;
    private static bool crouchKeys = true, cameraKey = true, manualKey = true, taps = true, cursorKnown = true, failed;

    public static void Reset()
    {
        Push = Brake = Left = Right = Crouch = Jump = Manual = false;
        EndStep(); JumpTap = ToggleTap = CameraTap = CrouchTap = false;
    }

    public static void Poll()
    {
        JumpTap = ToggleTap = CameraTap = CrouchTap = false;
        if (Scripted || failed) return;
        try
        {
            var kb = Keyboard.current;
            if (kb == null || CursorFree()) { Push = Brake = Left = Right = Crouch = Jump = Manual = false; wasPush = wasBrake = wasLeft = wasRight = wasJump = wasCrouch = wasToggle = wasCamera = false; EndStep(); return; }
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
            if (push && !wasPush) FlipUp = true;
            if (brake && !wasBrake) FlipDown = true;
            if (left && !wasLeft) LeftPressed = true;
            if (right && !wasRight) RightPressed = true;
            if (jump && !wasJump) { JumpPressed = true; JumpTap = true; }
            if (crouch && !wasCrouch) CrouchTap = true;
            if (toggle && !wasToggle) ToggleTap = true;
            if (camera && !wasCamera) CameraTap = true;
            Push = push; Brake = brake; Left = left; Right = right; Jump = jump; Crouch = crouch; Manual = manual;
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
