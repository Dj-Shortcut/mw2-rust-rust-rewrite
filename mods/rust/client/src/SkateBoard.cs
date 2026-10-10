// Rust/Standard is the game's own lit shader: a plugin-made mesh drawn with it takes the world's
// light, casts a shadow and goes dark at night. It reads colours from a texture, not from the
// vertices. Hidden/Internal-Colored draws vertex colours without light and is the fallback.
// The mesh setters can be missing from a client build; primitives are the fallback for those.
using System;
using Il2CppInterop.Runtime;
using UnityEngine;

public static class SkateBoard
{
    public const float TipDegrees = 15f, Axle = 0.23f;
    public const string LitShader = "Rust/Standard", PlainShader = "Hidden/Internal-Colored";
    public const float Gloss = 0.2f;
    public static string Drawn = "";
    public static float ShowLift, Tip;
    private static void Say(string m) { Out.Say("SKATE " + m); }

    public static GameObject Build()
    {
        try { var made = FromMesh(); Say("BOARD from the shared mesh"); return made; }
        catch (Exception e) { Say("shared mesh unavailable (" + e.GetType().Name + ": " + e.Message + "); using primitives"); return FromPrimitives(); }
    }

    public static Vector3 Contact(Transform playerT) { return playerT.position + Vector3.up * ShowLift; }

    // A manual lifts the end that leads: the board, and the rider's footing with it, pitches about
    // the ground under the trailing axle.
    public static void Tilt(ref Vector3 at, ref Vector3 up)
    {
        if (Tip < 0.001f) return;
        var lead = SkateRide.Dir(SkateRide.Yaw);
        lead = lead - up * Vector3.Dot(lead, up);
        if (lead.sqrMagnitude < 0.0001f) return;
        lead = SkateRide.Speed >= 0f ? lead.normalized : -lead.normalized;
        var turn = SkateRide.Turn(Tip * TipDegrees, Vector3.Cross(lead, up));
        var pivot = at - lead * Axle;
        at = pivot + turn * (at - pivot);
        up = turn * up;
    }

    public static void Follow(GameObject board, Transform playerT)
    {
        var show = SkateRide.On && playerT != null;
        if (board.activeSelf != show) board.SetActive(show);
        if (!show) return;
        var n = SkateRide.ViewNormal;
        var at = Contact(playerT);
        Tilt(ref at, ref n);
        var travel = SkateRide.Dir(SkateRide.Yaw);
        travel = travel - n * Vector3.Dot(travel, n);
        travel = travel.sqrMagnitude > 0.0001f ? travel.normalized : SkateRide.Dir(SkateRide.Yaw);
        // The lean rolls the board about its direction of travel, right side down in a right turn.
        var up = SkateRide.Turn(-SkateRide.Lean, travel) * n;
        var nose = SkateRide.Dir(SkateRide.NoseYaw);
        nose = nose - up * Vector3.Dot(nose, up);
        nose = nose.sqrMagnitude > 0.0001f ? nose.normalized : travel;
        var flip = (float)SkateRide.FlipDeg;
        if (SkateRide.Mode == RideMode.Bail)
        {
            var t = Time.realtimeSinceStartup - SkateRide.ModeAt;
            flip = t * 540f;
            at = at + travel * (t * 1.6f) + Vector3.up * (0.25f * (float)Math.Sin(Math.Min(1f, t / SkateRide.BailSeconds) * Math.PI));
        }
        var rot = Quaternion.LookRotation(nose, up) * SkateRide.Turn(flip, Vector3.forward);
        // A flip turns the board about the middle of the deck, not about the wheels.
        board.transform.rotation = rot;
        board.transform.position = at + up * RiderRig.DeckTop - (rot * Vector3.up) * RiderRig.DeckTop;
    }

    private static GameObject FromMesh()
    {
        var data = Shortcut.RustMod.SkateBoardMesh.Create();
        var n = data.Positions.Length / 3;
        var verts = new Vector3[n]; var norms = new Vector3[n]; var cols = new Color[n];
        for (var i = 0; i < n; i++)
        {
            verts[i] = new Vector3(data.Positions[i * 3], data.Positions[i * 3 + 1], data.Positions[i * 3 + 2]);
            norms[i] = new Vector3(data.Normals[i * 3], data.Normals[i * 3 + 1], data.Normals[i * 3 + 2]);
            cols[i] = new Color(data.Colours[i * 4], data.Colours[i * 4 + 1], data.Colours[i * 4 + 2], data.Colours[i * 4 + 3]);
        }
        var mesh = new Mesh();
        mesh.vertices = verts; mesh.normals = norms; mesh.colors = cols; mesh.triangles = data.Triangles;
        mesh.RecalculateBounds();
        var mat = Lit(mesh, cols);
        if (mat == null)
        {
            var sh = Shader.Find(PlainShader);
            if (sh == null) throw new InvalidOperationException("no vertex-colour shader");
            mat = new Material(sh); mat.color = Color.white; Drawn = PlainShader;
        }
        var root = new GameObject("skate_board");
        root.AddComponent<MeshFilter>().sharedMesh = mesh;
        root.AddComponent<MeshRenderer>().material = mat;
        UnityEngine.Object.DontDestroyOnLoad(root);
        return root;
    }

    private static Shader Find(string name, out string how)
    {
        how = "by name";
        var sh = Shader.Find(name);
        if (sh != null) return sh;
        // A shader that came with an asset bundle is loaded without being findable by name.
        how = "among the loaded shaders";
        var all = Resources.FindObjectsOfTypeAll(Il2CppType.Of<Shader>());
        for (var i = 0; i < all.Length; i++)
        {
            var one = all[i] == null ? null : all[i].TryCast<Shader>();
            if (one != null && one.name == name) return one;
        }
        return null;
    }

    // One block of texels per vertex colour; every vertex looks its own colour up.
    private static Material Lit(Mesh mesh, Color[] cols)
    {
        try
        {
            string how;
            var sh = Find(LitShader, out how);
            if (sh == null) { Say("no " + LitShader + " shader in this client; the board is drawn without light"); return null; }
            var unique = new System.Collections.Generic.List<Color>(); var index = new int[cols.Length];
            for (var i = 0; i < cols.Length; i++)
            {
                var c = cols[i]; var found = -1;
                for (var u = 0; u < unique.Count && found < 0; u++)
                    if (Math.Abs(unique[u].r - c.r) + Math.Abs(unique[u].g - c.g) + Math.Abs(unique[u].b - c.b) < 0.002f) found = u;
                if (found < 0) { found = unique.Count; unique.Add(c); }
                index[i] = found;
            }
            const int cell = 4;
            var across = 1; while (across * across < unique.Count) across++;
            var tex = new Texture2D(cell * across, cell * across);
            for (var u = 0; u < across * across; u++)
            {
                var c = u < unique.Count ? unique[u] : Color.white;
                for (var y = 0; y < cell; y++) for (var x = 0; x < cell; x++) tex.SetPixel((u % across) * cell + x, (u / across) * cell + y, c);
            }
            tex.filterMode = FilterMode.Point;
            tex.Apply();
            var uv = new Vector2[cols.Length];
            for (var i = 0; i < uv.Length; i++) uv[i] = new Vector2(((index[i] % across) + 0.5f) / across, ((index[i] / across) + 0.5f) / across);
            mesh.uv = uv;
            var m = new Material(sh);
            m.mainTexture = tex; m.color = Color.white;
            m.SetFloat("_Glossiness", Gloss); m.SetFloat("_Metallic", 0f);
            Drawn = LitShader;
            Say("BOARD lit by " + LitShader + " (found " + how + "), " + unique.Count + " colours");
            return m;
        }
        catch (Exception e) { Say("lit board unavailable (" + e.GetType().Name + ": " + e.Message + "); it is drawn without light"); return null; }
    }

    private static GameObject FromPrimitives()
    {
        var root = new GameObject("skate_board");
        var sh = Shader.Find(PlainShader); Drawn = PlainShader + ", primitives";
        var grip = new Color(0.07f, 0.07f, 0.08f); var kick = new Color(0.16f, 0.16f, 0.18f); var under = new Color(1f, 0.1f, 0.6f);
        var metal = new Color(0.55f, 0.57f, 0.6f); var wheel = new Color(0.95f, 0.93f, 0.85f);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.078f, 0f), new Vector3(0.20f, 0.010f, 0.60f), Quaternion.identity, grip, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.069f, 0f), new Vector3(0.20f, 0.008f, 0.60f), Quaternion.identity, under, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.084f, 0f), new Vector3(0.025f, 0.003f, 0.56f), Quaternion.identity, under, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.097f, 0.365f), new Vector3(0.20f, 0.012f, 0.15f), Quaternion.Euler(-17f, 0f, 0f), kick, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.097f, -0.365f), new Vector3(0.20f, 0.012f, 0.15f), Quaternion.Euler(17f, 0f, 0f), kick, sh);
        for (var z = -1; z <= 1; z += 2)
        {
            Part(root, PrimitiveType.Cube, new Vector3(0f, 0.046f, z * 0.23f), new Vector3(0.15f, 0.034f, 0.05f), Quaternion.identity, metal, sh);
            for (var x = -1; x <= 1; x += 2)
                Part(root, PrimitiveType.Cylinder, new Vector3(x * 0.097f, 0.0275f, z * 0.23f), new Vector3(0.055f, 0.017f, 0.055f), Quaternion.Euler(0f, 0f, 90f), wheel, sh);
        }
        UnityEngine.Object.DontDestroyOnLoad(root);
        return root;
    }

    private static void Part(GameObject root, PrimitiveType type, Vector3 pos, Vector3 scale, Quaternion rot, Color color, Shader sh)
    {
        var go = GameObject.CreatePrimitive(type);
        var col = go.GetComponent<Collider>();
        if (col != null) UnityEngine.Object.Destroy(col);
        go.transform.SetParent(root.transform, false);
        go.transform.localPosition = pos; go.transform.localRotation = rot; go.transform.localScale = scale;
        if (sh != null) { var m = new Material(sh); m.color = color; go.GetComponent<Renderer>().material = m; }
    }
}
