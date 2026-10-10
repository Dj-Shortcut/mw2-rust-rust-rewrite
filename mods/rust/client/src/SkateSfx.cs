using System;
using Shortcut.RustMod;
using UnityEngine;

public static class SkateSfx
{
    public const int Rate = 44100;
    public static bool Ready;
    public static string State = "not started";
    private static AudioSource roll, grind, shots;
    private static AudioClip push, ollie, landing, bail;
    private static int pops, lands, bails, pushes;
    private static float rollVolume, grindVolume;
    private static bool failed;

    private static AudioClip Clip(SkateSound sound, uint seed)
    {
        float[] samples; string error;
        if (!SkateAudio.TryCreate(sound, Rate, seed, out samples, out error)) throw new InvalidOperationException(sound + ": " + error);
        var clip = AudioClip.Create("skate_" + sound, samples.Length, 1, Rate, false);
        clip.SetData(samples, 0);
        return clip;
    }

    private static AudioSource Source(GameObject go, AudioClip clip, bool loop)
    {
        var s = go.AddComponent<AudioSource>();
        s.spatialBlend = 0f; s.loop = loop; s.volume = loop ? 0f : 1f;
        if (clip != null) { s.clip = clip; s.Play(); }
        return s;
    }

    public static void Start()
    {
        if (Ready || failed) return;
        try
        {
            var go = new GameObject("skate_audio");
            UnityEngine.Object.DontDestroyOnLoad(go);
            roll = Source(go, Clip(SkateSound.Rolling, 11u), true);
            grind = Source(go, Clip(SkateSound.Grind, 12u), true);
            shots = Source(go, null, false);
            push = Clip(SkateSound.Push, 13u); ollie = Clip(SkateSound.Ollie, 14u); landing = Clip(SkateSound.Landing, 15u); bail = Clip(SkateSound.Bail, 16u);
            Sync();
            Ready = true; State = "ready";
            Out.Say("AUDIO ready: two loops and four one-shots at " + Rate + " Hz");
        }
        catch (Exception e) { failed = true; State = "failed: " + e.GetType().Name + ": " + e.Message; Out.Say("AUDIO " + State); }
    }

    private static void Sync() { pops = SkateRide.Pops; lands = SkateRide.Lands; bails = SkateRide.Bails; pushes = SkateRide.Pushes; }

    public static void Update(float frameSeconds)
    {
        if (!Ready || failed) return;
        try
        {
            var speed = Math.Abs(SkateRide.Speed);
            var wantRoll = SkateRide.Mode == RideMode.Ground ? SkateRide.Clamp(speed / 9f, 0f, 1f) * 0.55f : 0f;
            var wantGrind = SkateRide.Mode == RideMode.Grind ? 0.6f : 0f;
            var k = SkateRide.Clamp(frameSeconds * 12f, 0f, 1f);
            rollVolume += (wantRoll - rollVolume) * k; grindVolume += (wantGrind - grindVolume) * k;
            roll.volume = rollVolume; roll.pitch = 0.75f + speed / 13f * 0.7f;
            grind.volume = grindVolume; grind.pitch = 0.85f + speed / 13f * 0.4f;
            if (!SkateRide.On) { Sync(); return; }
            if (SkateRide.Pushes != pushes) shots.PlayOneShot(push, 0.5f);
            if (SkateRide.Pops != pops) shots.PlayOneShot(ollie, 0.9f);
            if (SkateRide.Lands != lands) shots.PlayOneShot(landing, SkateRide.Clamp(0.35f + SkateRide.LandImpact / 10f, 0.35f, 1f));
            if (SkateRide.Bails != bails) shots.PlayOneShot(bail, 1f);
            Sync();
        }
        catch (Exception e) { failed = true; State = "failed while playing: " + e.GetType().Name + ": " + e.Message; Out.Say("AUDIO " + State); }
    }
}
