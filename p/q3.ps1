# Survey for the movement probe: what is usable on the walk component, and whether the new Input
# System types are present in the interop set (the legacy Input class is disabled in this build).
$global:pfind = $null; $global:pp = $null
$global:pt = 'PlayerWalkMovement,BaseMovement,UnityEngine.InputSystem.Keyboard~^(current|wKey|aKey|sKey|dKey|spaceKey|leftShiftKey|kKey|anyKey)$,UnityEngine.InputSystem.Gamepad~^(current|leftStick|rightStick|buttonSouth|buttonEast|rightTrigger|leftTrigger)$,UnityEngine.InputSystem.Controls.ButtonControl~^(isPressed|wasPressedThisFrame|wasReleasedThisFrame)$,UnityEngine.InputSystem.Controls.StickControl~^(x|y|up|down|left|right)$,UnityEngine.InputSystem.Controls.Vector2Control~^(x|y)$'
zz api
