// A way for the plugin to run a console command. The client's own console API is obfuscated, but
// the console's input box is an ordinary TextMeshPro input field (EngineUI2_ConsoleInput, hint
// "Enter Command"), so a command can be put into it and its submit event raised. Whether the game
// acts on that event is checked by the pose scenario; nothing depends on it yet.
using System;
using Il2CppInterop.Runtime;
using UnityEngine;

public static class GameConsole
{
    private static TMPro.TMP_InputField field;

    public static string Send(string command, bool throughEndEdit)
    {
        try
        {
            if (field == null) field = Find();
            if (field == null) return "no console input field";
            field.text = command;
            if (throughEndEdit) field.onEndEdit.Invoke(command); else field.onSubmit.Invoke(command);
            return "'" + command + "' raised through " + (throughEndEdit ? "onEndEdit" : "onSubmit") + " of " + field.gameObject.name;
        }
        catch (Exception e) { return "threw " + e.GetType().Name + ": " + e.Message; }
    }

    private static TMPro.TMP_InputField Find()
    {
        var all = Resources.FindObjectsOfTypeAll(Il2CppType.Of<TMPro.TMP_InputField>());
        for (var i = 0; i < all.Length; i++)
        {
            var f = all[i].TryCast<TMPro.TMP_InputField>();
            if (f != null && f.gameObject.name.Contains("ConsoleInput")) return f;
        }
        return null;
    }
}
