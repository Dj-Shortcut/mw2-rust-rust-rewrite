using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using Newtonsoft.Json;
using Oxide.Core;
using Rust;

namespace Oxide.Plugins
{
    [Info("ShortcutLoadouts", "Dj-Shortcut", "0.1.0")]
    [Description("Permission-based personal firearm kits and bounded Bullet PvP damage.")]
    public sealed class ShortcutLoadouts : RustPlugin
    {
        private const string UsePermission = "shortcutloadouts.use";
        private const string DamagePermission = "shortcutloadouts.damage";
        private const AmmoTypes FirearmAmmo = AmmoTypes.PISTOL_9MM | AmmoTypes.RIFLE_556MM |
            AmmoTypes.SHOTGUN_12GUAGE | AmmoTypes.HANDMADE_SHELL;
        private readonly Stopwatch _clock = Stopwatch.StartNew();
        private readonly HashSet<ulong> _granting = new HashSet<ulong>();
        private readonly Dictionary<ulong, double> _nextGrant = new Dictionary<ulong, double>();
        private readonly Dictionary<string, List<ResolvedEntry>> _kits =
            new Dictionary<string, List<ResolvedEntry>>(StringComparer.Ordinal);
        private readonly HashSet<uint> _firearmPrefabs = new HashSet<uint>();
        private Settings _settings;
        private bool _ready;

        public sealed class Settings
        {
            [JsonProperty(Required = Required.Always)] public int Version;
            [JsonProperty(Required = Required.Always)] public int CooldownSeconds;
            [JsonProperty(Required = Required.Always)] public double BulletDamageFactor;
            [JsonProperty(Required = Required.Always)] public Dictionary<string, List<KitEntry>> Loadouts;
        }

        public sealed class KitEntry
        {
            [JsonProperty(Required = Required.Always)] public string Shortname;
            [JsonProperty(Required = Required.Always)] public int Amount;
        }

        private sealed class ResolvedEntry
        {
            public ItemDefinition Definition;
            public int Amount;
        }

        private sealed class Delivery
        {
            public ResolvedEntry Entry;
            public ItemContainer Container;
            public int Slot;
            public Item Item;
            public ulong Uid;
        }

        private static Settings DefaultSettings()
        {
            return new Settings
            {
                Version = 1, CooldownSeconds = 60, BulletDamageFactor = 1,
                Loadouts = new Dictionary<string, List<KitEntry>>
                {
                    { "carbine", new List<KitEntry>
                        {
                            new KitEntry { Shortname = "rifle.ak", Amount = 1 },
                            new KitEntry { Shortname = "ammo.rifle", Amount = 120 }
                        }
                    }
                }
            };
        }

        protected override void LoadDefaultConfig()
        {
            Config.WriteObject(DefaultSettings(), true);
        }

        protected override void LoadConfig()
        {
            _ready = false;
            _settings = null;
            try
            {
                // Oxide creates defaults only when the configuration file is absent.
                base.LoadConfig();
                _settings = ParseSettings(File.ReadAllText(Config.Filename));
            }
            catch (Exception error)
            {
                PrintError("Configuration rejected; both features are disabled. Existing file is preserved. " +
                    error.Message);
            }
        }

        internal static Settings ParseSettings(string json)
        {
            var settings = JsonConvert.DeserializeObject<Settings>(json, new JsonSerializerSettings
            {
                TypeNameHandling = TypeNameHandling.None,
                MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
                MissingMemberHandling = MissingMemberHandling.Error
            });
            string error = ValidateSettings(settings);
            if (error != null) throw new InvalidDataException(error);
            return settings;
        }

        private void Init()
        {
            permission.RegisterPermission(UsePermission, this);
            permission.RegisterPermission(DamagePermission, this);
        }

        private void OnServerInitialized()
        {
            _ready = false;
            _kits.Clear();
            _firearmPrefabs.Clear();
            if (_settings == null) return;
            try
            {
                foreach (ItemDefinition definition in ItemManager.itemList)
                {
                    BaseProjectile firearm = Firearm(definition);
                    if (firearm != null) _firearmPrefabs.Add(firearm.prefabID);
                }
                foreach (var kit in _settings.Loadouts)
                {
                    var entries = new List<ResolvedEntry>();
                    foreach (KitEntry entry in kit.Value)
                    {
                        ItemDefinition definition = ItemManager.FindItemDefinition(entry.Shortname);
                        if (definition == null || entry.Amount > definition.stackable ||
                            !SupportedItem(definition))
                            throw new InvalidDataException("Unsupported item or amount in loadout " + kit.Key +
                                ": " + entry.Shortname);
                        entries.Add(new ResolvedEntry { Definition = definition, Amount = entry.Amount });
                    }
                    _kits.Add(kit.Key, entries);
                }
                _ready = true;
            }
            catch (Exception error)
            {
                _kits.Clear();
                _firearmPrefabs.Clear();
                PrintError("Item configuration rejected; both features are disabled. " + error.Message);
            }
        }

        private static BaseProjectile Firearm(ItemDefinition definition)
        {
            ItemModEntity entity = definition.GetComponent<ItemModEntity>();
            BaseProjectile firearm = entity == null || entity.entityPrefab == null ? null :
                entity.entityPrefab.GetEntity() as BaseProjectile;
            return firearm != null && firearm.primaryMagazine != null &&
                (firearm.primaryMagazine.definition.ammoTypes & FirearmAmmo) != 0 ? firearm : null;
        }

        private static bool SupportedItem(ItemDefinition definition)
        {
            if (definition.category == ItemCategory.Weapon) return Firearm(definition) != null;
            if (definition.category != ItemCategory.Ammunition) return false;
            ItemModProjectile ammunition = definition.GetComponent<ItemModProjectile>();
            return ammunition != null && (ammunition.ammoType & FirearmAmmo) != 0;
        }

        internal static string ValidateSettings(Settings settings)
        {
            if (settings == null || settings.Version != 1) return "Version must be 1.";
            if (settings.CooldownSeconds < 1 || settings.CooldownSeconds > 3600)
                return "CooldownSeconds must be between 1 and 3600.";
            if (!IsFinite(settings.BulletDamageFactor) || settings.BulletDamageFactor < 0.1 ||
                settings.BulletDamageFactor > 4) return "BulletDamageFactor must be between 0.1 and 4.";
            if (settings.Loadouts == null || settings.Loadouts.Count < 1 || settings.Loadouts.Count > 16)
                return "Configure between 1 and 16 loadouts.";
            foreach (var kit in settings.Loadouts)
            {
                if (string.IsNullOrEmpty(kit.Key) || kit.Key.Length > 32 ||
                    kit.Key.Any(c => !(c >= 'a' && c <= 'z') && !(c >= '0' && c <= '9') && c != '-' && c != '_'))
                    return "Loadout names must use lowercase letters, digits, hyphens or underscores.";
                if (kit.Value == null || kit.Value.Count < 1 || kit.Value.Count > 24)
                    return "Each loadout needs between 1 and 24 entries.";
                foreach (KitEntry entry in kit.Value)
                    if (entry == null || string.IsNullOrEmpty(entry.Shortname) || entry.Shortname.Length > 64 ||
                        entry.Shortname.Any(c => !(c >= 'a' && c <= 'z') && !(c >= '0' && c <= '9') && c != '.') ||
                        entry.Amount < 1 || entry.Amount > 2048)
                        return "Each entry needs a valid shortname and an amount between 1 and 2048.";
            }
            return null;
        }

        internal static bool IsFinite(double value)
        {
            return !double.IsNaN(value) && !double.IsInfinity(value);
        }

        internal static bool Eligible(bool connected, bool npc, bool alive, bool sleeping, bool wounded)
        {
            return connected && !npc && alive && !sleeping && !wounded;
        }

        private bool CanGrant(BasePlayer player)
        {
            return _ready && player != null &&
                Eligible(player.IsConnected, player.IsNpc, player.IsAlive(), player.IsSleeping(), player.IsWounded()) &&
                permission.UserHasPermission(player.UserIDString, UsePermission);
        }

        [ChatCommand("loadout")]
        private void LoadoutCommand(BasePlayer player, string command, string[] args)
        {
            if (player == null) return;
            if (!_ready) { SendReply(player, "Loadouts are disabled. Ask the server administrator."); return; }
            if (!permission.UserHasPermission(player.UserIDString, UsePermission))
            { SendReply(player, "You do not have permission to use loadouts."); return; }
            if (!CanGrant(player))
            { SendReply(player, "You must be connected, alive, awake and not wounded to claim a loadout."); return; }
            if (args == null || args.Length != 1 || !_kits.ContainsKey(args[0]))
            { SendReply(player, "Use /loadout <name>. Available: " + string.Join(", ", _kits.Keys.OrderBy(k => k))); return; }
            if (_granting.Contains(player.userID))
            { SendReply(player, "A loadout request is already in progress."); return; }
            double now = _clock.Elapsed.TotalSeconds;
            double next;
            if (_nextGrant.TryGetValue(player.userID, out next) && next > now)
            { SendReply(player, "Wait " + Math.Ceiling(next - now) + " seconds before claiming another loadout."); return; }
            foreach (ulong expired in _nextGrant.Where(pair => pair.Value <= now).Select(pair => pair.Key).ToArray())
                _nextGrant.Remove(expired);
            if (_nextGrant.Count >= 4096)
            { SendReply(player, "Loadouts are temporarily busy. Please try again later."); return; }

            _granting.Add(player.userID);
            var deliveries = new List<Delivery>();
            bool complete = false;
            try
            {
                Reserve(player, _kits[args[0]], deliveries);
                foreach (Delivery delivery in deliveries)
                {
                    if (!CanGrant(player)) throw new InvalidOperationException("Player state changed.");
                    delivery.Item = ItemManager.CreateByName(delivery.Entry.Definition.shortname, delivery.Entry.Amount);
                    if (delivery.Item == null) throw new InvalidOperationException("Item creation failed.");
                    delivery.Uid = delivery.Item.uid.Value;
                    if (delivery.Uid == 0 || delivery.Item.info != delivery.Entry.Definition ||
                        delivery.Item.amount != delivery.Entry.Amount || delivery.Item.IsRemoved())
                        throw new InvalidOperationException("Created item changed.");
                }
                foreach (Delivery delivery in deliveries)
                {
                    if (!CanGrant(player) || !OwnsContainer(player, delivery.Container) ||
                        !EmptySlot(delivery.Container, delivery.Slot, delivery.Entry.Amount) ||
                        !MatchesCreated(delivery) || delivery.Container.SlotTaken(delivery.Item, delivery.Slot))
                        throw new InvalidOperationException("Reserved slot or item changed.");
                    if (!delivery.Item.MoveToContainer(delivery.Container, delivery.Slot, false, false, player, false) ||
                        !OwnsContainer(player, delivery.Container) || !Delivered(delivery))
                        throw new InvalidOperationException("Item delivery failed.");
                }
                if (!CanGrant(player) || deliveries.Any(delivery =>
                    !OwnsContainer(player, delivery.Container) || !Delivered(delivery)))
                    throw new InvalidOperationException("Final delivery verification failed.");
                _nextGrant[player.userID] = _clock.Elapsed.TotalSeconds + _settings.CooldownSeconds;
                complete = true;
            }
            catch (Exception error)
            {
                if (!(error is InvalidOperationException)) PrintWarning("Loadout grant failed: " + error.Message);
            }
            finally
            {
                // Retain the reentrancy guard while removal hooks run.
                try { if (!complete) Rollback(deliveries); }
                finally { _granting.Remove(player.userID); }
            }
            SendReply(player, complete ? "Loadout claimed: " + args[0] + "." :
                "Loadout could not be delivered. Keep enough empty main or belt slots and try again.");
        }

        private static void Reserve(BasePlayer player, List<ResolvedEntry> kit, List<Delivery> deliveries)
        {
            foreach (ResolvedEntry entry in kit)
            {
                bool reserved = false;
                foreach (ItemContainer container in new[] { player.inventory.containerMain, player.inventory.containerBelt })
                {
                    if (container == null || container.capacity < 1 || container.capacity > 128) continue;
                    for (int slot = 0; slot < container.capacity; slot++)
                    {
                        if (!EmptySlot(container, slot, entry.Amount) ||
                            deliveries.Any(d => d.Container == container && d.Slot == slot)) continue;
                        deliveries.Add(new Delivery { Entry = entry, Container = container, Slot = slot });
                        reserved = true;
                        break;
                    }
                    if (reserved) break;
                }
                if (!reserved) throw new InvalidOperationException("Not enough empty inventory slots.");
            }
        }

        internal static bool AmountFits(int amount, int containerLimit)
        {
            return amount > 0 && (containerLimit <= 0 || amount <= containerLimit);
        }

        private static bool OwnsContainer(BasePlayer player, ItemContainer container)
        {
            return container != null && ReferenceEquals(container.playerOwner, player) &&
                player.inventory != null && (ReferenceEquals(container, player.inventory.containerMain) ||
                ReferenceEquals(container, player.inventory.containerBelt));
        }

        private static bool EmptySlot(ItemContainer container, int slot, int amount)
        {
            // Defined equipment slots can remove conflicting items even with swapping disabled.
            return container != null && !container.IsLocked() && !container.HasAvailableSlotsDefined &&
                slot >= 0 && slot < container.capacity && AmountFits(amount, container.maxStackSize) &&
                container.GetSlot(slot) == null && container.itemList.All(item => item.position != slot);
        }

        private static bool MatchesCreated(Delivery delivery)
        {
            return delivery.Item != null && delivery.Uid != 0 && delivery.Item.uid.Value == delivery.Uid &&
                !delivery.Item.IsRemoved() && delivery.Item.info == delivery.Entry.Definition &&
                delivery.Item.amount == delivery.Entry.Amount;
        }

        private static bool Delivered(Delivery delivery)
        {
            return MatchesCreated(delivery) && delivery.Item.parent == delivery.Container &&
                delivery.Item.position == delivery.Slot && delivery.Container.GetSlot(delivery.Slot) == delivery.Item;
        }

        private void Rollback(List<Delivery> deliveries)
        {
            foreach (Delivery delivery in deliveries)
            {
                Item item = delivery.Item;
                // An item object can be pooled/reused by another hook; never remove a different UID.
                if (item == null || delivery.Uid == 0 || item.uid.Value != delivery.Uid || item.IsRemoved()) continue;
                try
                {
                    item.RemoveFromWorld();
                    if (item.uid.Value != delivery.Uid || item.IsRemoved()) continue;
                    item.RemoveFromContainer();
                    if (item.uid.Value != delivery.Uid || item.IsRemoved()) continue;
                    item.Remove();
                    if (item.uid.Value != delivery.Uid) continue;
                    if (!item.IsRemoved()) throw new InvalidOperationException("Item removal was vetoed.");
                }
                catch (Exception error)
                {
                    _ready = false;
                    PrintError("Cleanup incomplete for created item " + delivery.Uid +
                        "; both features disabled. Check conflicting plugins. " + error.Message);
                }
            }
        }

        internal static bool TryScaleBullet(float bullet, double factor, out float scaled)
        {
            scaled = bullet;
            double value = bullet * factor;
            if (!IsFinite(bullet) || bullet <= 0 || !IsFinite(factor) || factor < 0.1 || factor > 4 ||
                !IsFinite(value) || value <= 0 || value > float.MaxValue) return false;
            scaled = (float)value;
            return scaled > 0;
        }

        private object OnEntityTakeDamage(BaseCombatEntity entity, HitInfo info)
        {
            if (!_ready || _settings.BulletDamageFactor == 1 || info == null || info.damageTypes == null) return null;
            BasePlayer victim = entity as BasePlayer;
            BasePlayer attacker = info.InitiatorPlayer;
            BaseProjectile weapon = info.WeaponPrefab as BaseProjectile;
            if (victim == null || attacker == null || victim == attacker || victim.IsNpc || attacker.IsNpc ||
                !victim.IsConnected || !attacker.IsConnected || weapon == null ||
                !_firearmPrefabs.Contains(weapon.prefabID) ||
                !permission.UserHasPermission(attacker.UserIDString, DamagePermission)) return null;
            float scaled;
            if (TryScaleBullet(info.damageTypes.Get(DamageType.Bullet), _settings.BulletDamageFactor, out scaled))
                info.damageTypes.Set(DamageType.Bullet, scaled);
            return null;
        }

        private void Unload()
        {
            _ready = false;
            _kits.Clear();
            _firearmPrefabs.Clear();
            _nextGrant.Clear();
            _granting.Clear();
        }
    }
}
