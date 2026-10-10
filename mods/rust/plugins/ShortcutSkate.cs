using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using HarmonyLib;
using Newtonsoft.Json;
using Oxide.Core.Plugins;
using Rust;
using Shortcut.RustMod;
using Unity.Collections;
using UnityEngine;

namespace Shortcut.RustMod
{
    public sealed class SkateSessionBounds
    {
        public const double MaximumBatchSeconds = 0.5;
        public bool Active { get; private set; }
        public double AirSeconds { get; private set; }
        private double started;
        private double lastSample;
        private double lastSupported;
        private double duration;

        public static bool Eligible(bool connected, bool npc, bool alive, bool sleeping,
                                    bool wounded, bool mounted, bool swimming)
        { return connected && !npc && alive && !sleeping && !wounded && !mounted && !swimming; }

        public bool Begin(double now, bool supported)
        {
            if (!Finite(now) || now < 0 || !supported || Active) return false;
            Active = true;
            started = lastSample = lastSupported = now;
            duration = AirSeconds = 0;
            return true;
        }

        public void End() { Active = false; AirSeconds = 0; }

        public bool Observe(double now, double dt, double horizontalPath, double verticalPath,
                            bool fullySupported, double maxSpeed, double maxAir)
        {
            if (!Active) return false;
            if (!Finite(now) || now < lastSample || now - lastSample > MaximumBatchSeconds ||
                !Finite(dt) || dt <= 0 || dt > MaximumBatchSeconds ||
                !Finite(horizontalPath) || horizontalPath < 0 ||
                !Finite(verticalPath) || verticalPath < 0 ||
                !Finite(maxSpeed) || maxSpeed <= 0 || !Finite(maxAir) || maxAir <= 0 ||
                horizontalPath > maxSpeed * dt || verticalPath > 40 * dt ||
                duration + dt > now - started + 0.125)
            { End(); return false; }

            double air = AirSeconds;
            if (!fullySupported || air > 0)
                air = Math.Max(air + dt, now - lastSupported);
            if (air > maxAir)
            { End(); return false; }
            duration += dt;
            lastSample = now;
            AirSeconds = fullySupported ? 0 : air;
            if (fullySupported) lastSupported = now;
            return true;
        }

        public static bool Finite(double value)
        { return !double.IsNaN(value) && !double.IsInfinity(value); }
    }
}

namespace Oxide.Plugins
{
    [Info("ShortcutSkate", "Dj-Shortcut", "0.1.0")]
    [Description("Player-scoped skateboard speed and short-airtime movement bounds.")]
    public sealed class ShortcutSkate : RustPlugin
    {
        private const string UsePermission = "shortcutskate.use";
        private static ShortcutSkate instance;
        private readonly Stopwatch clock = Stopwatch.StartNew();
        private readonly Dictionary<ulong, Session> sessions = new Dictionary<ulong, Session>();
        private Settings settings;
        private int groundMask;
        private long batch;
        private bool validating;
        private bool ready;

        public sealed class Settings
        {
            [JsonProperty(Required = Required.Always)] public int Version;
            [JsonProperty(Required = Required.Always)] public bool AllowEveryone;
            [JsonProperty(Required = Required.Always)] public double MaximumHorizontalSpeed;
            [JsonProperty(Required = Required.Always)] public double MaximumAirtime;
        }

        private sealed class Session
        {
            public BasePlayer Player;
            public readonly SkateSessionBounds Bounds = new SkateSessionBounds();
            public long Batch;
            public bool Allowed;
            public Vector3 LastEnd;
        }

        protected override void LoadDefaultConfig()
        {
            Config.WriteObject(new Settings { Version = 1, AllowEveryone = true,
                MaximumHorizontalSpeed = 14, MaximumAirtime = 1 }, true);
        }

        protected override void LoadConfig()
        {
            ready = false;
            settings = null;
            try
            {
                base.LoadConfig();
                settings = ParseSettings(File.ReadAllText(Config.Filename));
            }
            catch (Exception error)
            { PrintError("Skating is disabled; configuration is preserved. " + error.Message); }
        }

        public static Settings ParseSettings(string json)
        {
            var value = JsonConvert.DeserializeObject<Settings>(json, new JsonSerializerSettings
            {
                TypeNameHandling = TypeNameHandling.None,
                MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
                MissingMemberHandling = MissingMemberHandling.Error
            });
            if (value == null || value.Version != 1 ||
                !SkateSessionBounds.Finite(value.MaximumHorizontalSpeed) ||
                value.MaximumHorizontalSpeed < 1 || value.MaximumHorizontalSpeed > 50 ||
                !SkateSessionBounds.Finite(value.MaximumAirtime) ||
                value.MaximumAirtime < 0.1 || value.MaximumAirtime > 2)
                throw new InvalidDataException("Use Version 1, speed 1-50 m/s and airtime 0.1-2 seconds.");
            return value;
        }

        private void Init()
        {
            permission.RegisterPermission(UsePermission, this);
            instance = this;
            groundMask = LayerMask.GetMask("Terrain", "World", "Construction", "Deployed");
        }

        private void OnServerInitialized()
        {
            ready = settings != null && groundMask != 0 && Patched("ValidateMoves", typeof(MovePatch), false) &&
                Patched("AreSpeeding", typeof(SpeedPatch), true) && Patched("AreFlying", typeof(FlyPatch), true);
            if (!ready) PrintError("Skating is disabled: matching movement patches and valid config are required.");
            timer.Every(0.1f, Sweep);
        }

        private static bool Patched(string name, Type patch, bool postfix)
        {
            var method = AccessTools.Method(typeof(AntiHack), name);
            var info = method == null ? null : Harmony.GetPatchInfo(method);
            if (info == null) return false;
            return (postfix ? info.Postfixes : info.Prefixes).Any(p => p.PatchMethod.DeclaringType == patch) &&
                (postfix || info.Finalizers.Any(p => p.PatchMethod.DeclaringType == patch));
        }

        private bool Permitted(BasePlayer player)
        { return settings != null && (settings.AllowEveryone || permission.UserHasPermission(player.UserIDString, UsePermission)); }

        private static bool Eligible(BasePlayer player)
        {
            return player != null && SkateSessionBounds.Eligible(player.IsConnected, player.IsNpc,
                player.IsAlive(), player.IsSleeping(), player.IsWounded(), player.isMounted, player.IsSwimming()) &&
                player.GetParentEntity() == null;
        }

        [ConsoleCommand("skate.toggle")]
        private void Toggle(ConsoleSystem.Arg arg) { Command(arg, 0); }
        [ConsoleCommand("skate.on")]
        private void On(ConsoleSystem.Arg arg) { Command(arg, 1); }
        [ConsoleCommand("skate.off")]
        private void Off(ConsoleSystem.Arg arg) { Command(arg, -1); }
        [ConsoleCommand("skate.status")]
        private void Status(ConsoleSystem.Arg arg)
        {
            BasePlayer player = arg.Player();
            if (player == null) { arg.ReplyWith("Use this command as a player."); return; }
            Session session;
            bool active = sessions.TryGetValue(player.userID, out session) && ReferenceEquals(player, session.Player);
            if (active && (!Eligible(player) || !Permitted(player)))
            { Stop(player, "Skating off."); active = false; }
            arg.ReplyWith(active ? "Skating on." : "Skating off.");
        }

        private void Command(ConsoleSystem.Arg arg, int action)
        {
            BasePlayer player = arg.Player();
            if (player == null) { arg.ReplyWith("Use this command as a player."); return; }
            if (action < 0 || (action == 0 && sessions.ContainsKey(player.userID)))
            { if (!Stop(player, "Skating off.")) arg.ReplyWith("Skating off."); return; }
            if (!ready) { arg.ReplyWith("Skating is unavailable. Ask the server administrator."); return; }
            if (!Permitted(player)) { arg.ReplyWith("You do not have permission to skate."); return; }
            if (!Eligible(player) || !Supported(player.transform.position))
            { arg.ReplyWith("Stand on dry ground, alive and awake, to skate."); return; }
            if (sessions.ContainsKey(player.userID)) { arg.ReplyWith("Skating on."); return; }
            var session = new Session { Player = player, LastEnd = player.transform.position };
            if (!session.Bounds.Begin(clock.Elapsed.TotalSeconds, true)) return;
            sessions.Add(player.userID, session);
            SendReply(player, "Skating on.");
        }

        private bool Stop(BasePlayer player, string message)
        {
            if (ReferenceEquals(player, null)) return false;
            Session session;
            if (!sessions.TryGetValue(player.userID, out session)) return false;
            session.Bounds.End();
            session.Allowed = false;
            sessions.Remove(player.userID);
            if (player != null && player.IsConnected) SendReply(player, message);
            return true;
        }

        private void Sweep()
        {
            foreach (Session session in sessions.Values.ToArray())
                if (!ready || !Eligible(session.Player) || !Permitted(session.Player)) Stop(session.Player, "Skating off.");
        }

        private void OnPlayerDeath(BasePlayer player, HitInfo info) { Stop(player, "Skating off."); }
        private void OnPlayerSleep(BasePlayer player) { Stop(player, "Skating off."); }
        private void OnPlayerDisconnected(BasePlayer player, string reason) { Stop(player, "Skating off."); }
        private void OnEntityMounted(BaseMountable mountable, BasePlayer player) { Stop(player, "Skating off."); }
        private void OnPlayerWound(BasePlayer player, HitInfo info) { Stop(player, "Skating off."); }

        private void Unload()
        {
            ready = false;
            foreach (Session session in sessions.Values.ToArray()) Stop(session.Player, "Skating off.");
            if (ReferenceEquals(instance, this)) instance = null;
        }

        private bool Supported(Vector3 point)
        {
            RaycastHit hit;
            return Physics.Raycast(point + Vector3.up * 0.15f, Vector3.down, out hit, 0.3f,
                groundMask, QueryTriggerInteraction.Ignore) && hit.normal.y >= 0.70710677f;
        }

        private void BeginBatch(ref BasePlayer.PlayerServerStates.ReadOnly states, NativeArray<int>.ReadOnly indices)
        {
            if (!ready || validating || batch == long.MaxValue) { validating = false; return; }
            validating = true;
            batch++;
            double now = clock.Elapsed.TotalSeconds;
            foreach (int index in indices)
            {
                BasePlayer player = states.PlayerCache.Objects[index];
                Session session;
                if (player == null || !sessions.TryGetValue(player.userID, out session)) continue;
                session.Allowed = false;
                session.Batch = batch;
                if (!ReferenceEquals(player, session.Player) || player.ActivePlayerInd != index ||
                    !Eligible(player) || !Permitted(player) || states.CachedStates[index].IsSwimming)
                { Stop(player, "Skating off."); continue; }
                try
                {
                    var cache = states.TickCache;
                    int count = cache.Infos[index].Count;
                    if (count < 1 || count >= cache.BufferSize) { Stop(player, "Skating off: movement unavailable."); continue; }
                    int offset = checked(index * cache.BufferSize);
                    double dt = states.TickDeltaTime[index];
                    if (!SkateSessionBounds.Finite(dt) || dt <= 0 || dt > SkateSessionBounds.MaximumBatchSeconds)
                    { Stop(player, "Skating off: movement unavailable."); continue; }
                    Vector3 previous = cache.Segments[offset].point;
                    if (!Coordinate(previous) || (previous - session.LastEnd).sqrMagnitude > 0.0001f)
                    { Stop(player, "Skating off: movement changed."); continue; }
                    bool supported = Supported(previous);
                    double horizontal = 0, vertical = 0;
                    for (int s = 1; s <= count; s++)
                    {
                        Vector3 point = cache.Segments[offset + s].point;
                        if (!Coordinate(point)) { horizontal = double.NaN; break; }
                        double x = (double)point.x - previous.x, z = (double)point.z - previous.z;
                        horizontal += Math.Sqrt(x * x + z * z);
                        vertical += Math.Abs((double)point.y - previous.y);
                        if (horizontal > settings.MaximumHorizontalSpeed * dt || vertical > 40 * dt) break;
                        int samples = Math.Max(1, (int)Math.Ceiling((point - previous).magnitude / 0.1));
                        for (int sample = 1; sample <= samples; sample++)
                            supported = Supported(Vector3.Lerp(previous, point, (float)sample / samples)) && supported;
                        previous = point;
                    }
                    session.Allowed = session.Bounds.Observe(now, dt, horizontal,
                        vertical, supported, settings.MaximumHorizontalSpeed, settings.MaximumAirtime);
                    if (session.Allowed) session.LastEnd = previous;
                    if (!session.Allowed) Stop(player, "Skating off: movement bounds exceeded.");
                }
                catch (Exception)
                { Stop(player, "Skating off: movement unavailable."); }
            }
        }

        private static bool Coordinate(Vector3 value)
        {
            return SkateSessionBounds.Finite(value.x) && SkateSessionBounds.Finite(value.y) &&
                SkateSessionBounds.Finite(value.z) && Math.Abs(value.x) <= 100000 &&
                Math.Abs(value.y) <= 100000 && Math.Abs(value.z) <= 100000;
        }

        private void Filter(ref BasePlayer.PlayerServerStates.ReadOnly states,
                            NativeArray<int>.ReadOnly indices, NativeArray<bool> results)
        {
            if (!ready || !validating) return;
            for (int n = 0; n < indices.Length; n++)
            {
                int index = indices[n];
                BasePlayer player = states.PlayerCache.Objects[index];
                Session session;
                if (player != null && player.ActivePlayerInd == index &&
                    sessions.TryGetValue(player.userID, out session) && ReferenceEquals(player, session.Player) &&
                    session.Batch == batch && session.Allowed && Eligible(player) && Permitted(player)) results[n] = false;
            }
        }

        // The result arrays are consumed before penalties and position rejection.
        // Patch the completed batch, preserving every other anti-hack result.
        [AutoPatch]
        [HarmonyPatch(typeof(AntiHack), "ValidateMoves")]
        private static class MovePatch
        {
            private static void Prefix(ref BasePlayer.PlayerServerStates.ReadOnly __0, NativeArray<int>.ReadOnly __1)
            {
                if (instance == null) return;
                try { instance.BeginBatch(ref __0, __1); }
                catch (Exception) { instance.validating = false; instance.ready = false; }
            }
            private static Exception Finalizer(Exception __exception)
            { if (instance != null) instance.validating = false; return __exception; }
        }

        [AutoPatch]
        [HarmonyPatch(typeof(AntiHack), "AreSpeeding")]
        private static class SpeedPatch
        {
            private static void Postfix(ref BasePlayer.PlayerServerStates.ReadOnly __0,
                                        NativeArray<int>.ReadOnly __2, NativeArray<bool> __3)
            {
                if (instance == null) return;
                try { instance.Filter(ref __0, __2, __3); }
                catch (Exception) { instance.validating = false; instance.ready = false; }
            }
        }

        [AutoPatch]
        [HarmonyPatch(typeof(AntiHack), "AreFlying")]
        private static class FlyPatch
        {
            private static void Postfix(ref BasePlayer.PlayerServerStates.ReadOnly __0,
                                        NativeArray<int>.ReadOnly __3, NativeArray<bool> __4)
            {
                if (instance == null) return;
                try { instance.Filter(ref __0, __3, __4); }
                catch (Exception) { instance.validating = false; instance.ready = false; }
            }
        }
    }
}
