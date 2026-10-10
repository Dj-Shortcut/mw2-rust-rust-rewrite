#!/usr/bin/env bash
# First-boot bootstrap for a private Rust PC / Oxide test server (issues #266, #289).
# Target: Ubuntu 24.04 x86_64 with 16 GB RAM, for example a Hetzner CX43. Run once as root,
# normally from cloud-init. It installs the official dedicated server (Steam app 258550) and the
# matching stable Oxide release, starts the server EAC-disabled for the owner's RustClient.exe,
# and opens only SSH and the UDP game port.
#
# Optional environment:
#   OWNER_STEAMID  SteamID64 written as server owner (administrator)
#   ALLOW_FROM     IPv4 address or CIDR allowed on the game port (default "any")
#
# Progress: /var/log/rust-bootstrap.log, or run `rust-status` on the server.
set -Eeuo pipefail

LOG=/var/log/rust-bootstrap.log
STAGE_FILE=/var/lib/rust-bootstrap.stage
SRV=/srv/rust
ROOT=$SRV/server
IDENTITY=mod-demo
GAME_PORT=28015
QUERY_PORT=28017
RCON_PORT=28016
HOSTNAME_TEXT="Shortcut Private Development"
MIN_RAM_KB=${MIN_RAM_KB:-12582912}   # 12 GiB available, the Facepunch baseline
MIN_DISK_KB=${MIN_DISK_KB:-26214400} # 25 GiB free
OWNER_STEAMID=${OWNER_STEAMID:-}
ALLOW_FROM=${ALLOW_FROM:-any}
STEAMCMD_URL=${STEAMCMD_URL:-https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz}
OXIDE_API=${OXIDE_API:-https://api.github.com/repos/OxideMod/Oxide.Rust/releases/latest}
OXIDE_FALLBACK_URL=${OXIDE_FALLBACK_URL:-https://github.com/OxideMod/Oxide.Rust/releases/latest/download/Oxide.Rust-linux.zip}
# Project plugin, pinned to a commit of this repository and checked against its SHA-256.
PLUGIN_URL=${PLUGIN_URL:-https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/fd97f10736c4f0749f07e799ee50442f2ddff01b/mods/rust/plugins/ShortcutLoadouts.cs}
PLUGIN_SHA256=${PLUGIN_SHA256:-ade96d31d5c03fc2adaa7f09e192211fb8511dcdb34539db8ceccac311bdf95e}

export HOME=/root DEBIAN_FRONTEND=noninteractive NEEDRESTART_MODE=a
mkdir -p "$(dirname "$STAGE_FILE")"
exec > >(tee -a "$LOG") 2>&1

stage() { printf '\n=== %s STAGE %s\n' "$(date -u +%H:%M:%S)" "$*"; printf '%s\n' "$*" > "$STAGE_FILE"; }
die() { printf 'BOOTSTRAP FAILED: %s\n' "$*"; printf 'failed: %s\n' "$*" > "$STAGE_FILE"; exit 1; }
trap 'printf "BOOTSTRAP FAILED at line %s during: %s\n" "$LINENO" "$(cat "$STAGE_FILE" 2>/dev/null)"' ERR
as_rust() { runuser -u rust -- env HOME=/home/rust "$@"; }
retry() { # retry <attempts> <sleep-seconds> <command...>
  local attempts=$1 pause=$2 n; shift 2
  for ((n = 1; n <= attempts; n++)); do "$@" && return 0; echo "attempt $n/$attempts failed: $*" >&2; sleep "$pause"; done
  return 1
}

stage "preflight"
[[ $EUID -eq 0 ]] || die "run as root"
[[ $(uname -m) == x86_64 ]] || die "this host is $(uname -m); the Rust dedicated server needs x86_64 (Intel/AMD), not Arm"
[[ -z $OWNER_STEAMID || $OWNER_STEAMID =~ ^7656119[0-9]{10}$ ]] || die "OWNER_STEAMID is not a SteamID64"
[[ $ALLOW_FROM == any || $ALLOW_FROM =~ ^[0-9]{1,3}(\.[0-9]{1,3}){3}(/[0-9]{1,2})?$ ]] || die "ALLOW_FROM must be 'any', an IPv4 address or an IPv4 CIDR"
ram_kb=$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)
(( ram_kb >= MIN_RAM_KB )) || die "only $((ram_kb / 1048576)) GiB RAM available; a 16 GB server is needed"
mkdir -p /srv
disk_kb=$(df --output=avail -k /srv | tail -n 1 | tr -d ' ')
(( disk_kb >= MIN_DISK_KB )) || die "only $((disk_kb / 1048576)) GiB disk free; 25 GiB is needed"
echo "ram available: $((ram_kb / 1048576)) GiB, disk free: $((disk_kb / 1048576)) GiB"

if [[ -x $ROOT/RustDedicated && -f $SRV/.installed ]]; then
  echo "server already installed; skipping download steps"
else
  stage "packages"
  retry 8 20 apt-get -o DPkg::Lock::Timeout=600 update -qq
  retry 8 20 apt-get -o DPkg::Lock::Timeout=600 install -y -qq --no-install-recommends \
    lib32gcc-s1 lib32stdc++6 ca-certificates curl unzip python3 ufw iproute2

  stage "account and directories"
  id rust >/dev/null 2>&1 || useradd --system --create-home --home-dir /home/rust --shell /usr/sbin/nologin rust
  install -d -o rust -g rust -m 0750 "$SRV" "$SRV/steamcmd" "$SRV/logs"

  stage "steamcmd"
  work=$(mktemp -d)
  chmod 0755 "$work"
  retry 5 15 curl -fsSL "$STEAMCMD_URL" -o "$work/steamcmd_linux.tar.gz"
  as_rust tar -xzf "$work/steamcmd_linux.tar.gz" -C "$SRV/steamcmd"

  stage "rust dedicated server download (several GB)"
  installed=0
  for attempt in 1 2 3 4 5; do
    echo "steamcmd attempt $attempt"
    as_rust "$SRV/steamcmd/steamcmd.sh" +force_install_dir "$ROOT" +login anonymous +app_update 258550 validate +quit || true
    if [[ -x $ROOT/RustDedicated ]] && grep -Eq '"StateFlags"[[:space:]]+"4"' "$ROOT/steamapps/appmanifest_258550.acf" 2>/dev/null; then
      installed=1
      break
    fi
    sleep 15
  done
  (( installed == 1 )) || die "SteamCMD did not finish installing app 258550"
  grep -E '"buildid"' "$ROOT/steamapps/appmanifest_258550.acf" || true
  if [[ -f $SRV/steamcmd/linux64/steamclient.so ]]; then
    as_rust mkdir -p /home/rust/.steam/sdk64
    as_rust cp "$SRV/steamcmd/linux64/steamclient.so" /home/rust/.steam/sdk64/steamclient.so
  fi

  stage "oxide"
  oxide_zip=$work/Oxide.Rust-linux.zip
  oxide_line=""
  if meta=$(retry 3 10 curl -fsSL -H 'Accept: application/vnd.github+json' "$OXIDE_API"); then
    oxide_line=$(python3 -c '
import json, re, sys
m = json.load(sys.stdin)
tag = m.get("tag_name", "")
assets = [a for a in m.get("assets", []) if a.get("name") == "Oxide.Rust-linux.zip"]
if m.get("draft") or m.get("prerelease") or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", tag) or len(assets) != 1:
    sys.exit(0)
url, digest = assets[0].get("browser_download_url", ""), assets[0].get("digest") or ""
expected = "https://github.com/OxideMod/Oxide.Rust/releases/download/%s/Oxide.Rust-linux.zip" % tag
if url == expected and re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
    print(tag, url, digest[7:])
' <<<"$meta" || true)
  fi
  if [[ -n $oxide_line ]]; then
    read -r oxide_tag oxide_url oxide_sha <<<"$oxide_line"
    echo "oxide release $oxide_tag"
    retry 5 15 curl -fsSL "$oxide_url" -o "$oxide_zip"
    echo "$oxide_sha  $oxide_zip" | sha256sum -c -
  else
    echo "WARNING: no usable release metadata; downloading the latest Oxide archive without a publisher digest check"
    retry 5 15 curl -fsSL "$OXIDE_FALLBACK_URL" -o "$oxide_zip"
  fi
  as_rust unzip -oq "$oxide_zip" -d "$ROOT"
  [[ -f $ROOT/RustDedicated_Data/Managed/Oxide.Rust.dll ]] || die "Oxide overlay did not produce Oxide.Rust.dll"

  stage "plugin and server identity"
  install -d -o rust -g rust "$ROOT/oxide" "$ROOT/oxide/plugins" "$ROOT/server" "$ROOT/server/$IDENTITY" "$ROOT/server/$IDENTITY/cfg"
  if retry 3 10 curl -fsSL "$PLUGIN_URL" -o "$work/ShortcutLoadouts.cs" && echo "$PLUGIN_SHA256  $work/ShortcutLoadouts.cs" | sha256sum -c -; then
    install -o rust -g rust -m 0644 "$work/ShortcutLoadouts.cs" "$ROOT/oxide/plugins/ShortcutLoadouts.cs"
  else
    echo "WARNING: ShortcutLoadouts.cs was not installed (download or checksum failed); the server still starts"
  fi
  if [[ -n $OWNER_STEAMID ]]; then
    printf 'ownerid %s "owner" "bootstrap"\n' "$OWNER_STEAMID" > "$ROOT/server/$IDENTITY/cfg/users.cfg"
    chown rust:rust "$ROOT/server/$IDENTITY/cfg/users.cfg"
    echo "owner set"
  fi
  rm -rf "$work"
  touch "$SRV/.installed"
fi

stage "launcher and service"
if [[ ! -s $SRV/rcon.secret ]]; then
  (umask 077; head -c 24 /dev/urandom | od -An -tx1 | tr -d ' \n' > "$SRV/rcon.secret")
  chown rust:rust "$SRV/rcon.secret"
fi
cat > "$SRV/start.sh" <<START
#!/usr/bin/env bash
# Starts the EAC-disabled development server. RCON listens on loopback only.
set -euo pipefail
cd "$ROOT"
export LD_LIBRARY_PATH="$ROOT/RustDedicated_Data/Plugins/x86_64:$ROOT/RustDedicated_Data/Plugins\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
exec ./RustDedicated -batchmode -nographics \\
  +server.port $GAME_PORT +server.queryport $QUERY_PORT \\
  +rcon.ip 127.0.0.1 +rcon.port $RCON_PORT +rcon.web 1 +rcon.password "\$(cat "$SRV/rcon.secret")" \\
  +server.identity "$IDENTITY" +server.level "Procedural Map" +server.worldsize 1500 +server.seed 12345 \\
  +server.maxplayers 4 +server.hostname "$HOSTNAME_TEXT" +app.port 1- \\
  -insecure -logfile "$SRV/logs/server.log"
START
chmod 0755 "$SRV/start.sh"

cat > /usr/local/bin/rust-status <<STATUS
#!/usr/bin/env bash
echo "bootstrap stage: \$(cat "$STAGE_FILE" 2>/dev/null)"
systemctl is-active rust-server 2>/dev/null | sed 's/^/service: /'
if ss -lunH | grep -q ":$GAME_PORT "; then echo "game port $GAME_PORT/udp: listening"; else echo "game port $GAME_PORT/udp: not listening yet"; fi
echo "--- bootstrap log"; tail -n 15 "$LOG" 2>/dev/null
echo "--- server log"; tail -n 15 "$SRV/logs/server.log" 2>/dev/null
STATUS
chmod 0755 /usr/local/bin/rust-status

cat > /etc/systemd/system/rust-server.service <<UNIT
[Unit]
Description=Rust dedicated server (private mod development, EAC disabled)
After=network-online.target
Wants=network-online.target
StartLimitIntervalSec=900
StartLimitBurst=4

[Service]
Type=simple
User=rust
Group=rust
WorkingDirectory=$ROOT
Environment=HOME=/home/rust
ExecStart=$SRV/start.sh
Restart=on-failure
RestartSec=20
KillSignal=SIGINT
TimeoutStopSec=120
LimitNOFILE=100000

[Install]
WantedBy=multi-user.target
UNIT

stage "firewall"
if [[ ${BOOTSTRAP_SKIP_FIREWALL:-0} == 1 ]]; then
  echo "firewall step skipped by BOOTSTRAP_SKIP_FIREWALL (test runs only)"
else
  ufw default deny incoming
  ufw default allow outgoing
  ufw allow 22/tcp
  if [[ $ALLOW_FROM == any ]]; then ufw allow "$GAME_PORT/udp"; else ufw allow proto udp from "$ALLOW_FROM" to any port "$GAME_PORT"; fi
  ufw --force enable
  ufw status verbose || true
fi

stage "start"
if [[ -d /run/systemd/system ]]; then
  systemctl daemon-reload
  systemctl enable --now rust-server.service
else
  echo "systemd is not running here; the service was written but not started"
fi

stage "waiting for the first map generation and server startup"
ready=0
for ((i = 0; i < 240; i++)); do
  if ss -lunH | grep -q ":$GAME_PORT "; then ready=1; break; fi
  if [[ -d /run/systemd/system ]] && ! systemctl is-active --quiet rust-server.service && (( i > 3 )); then
    echo "service is not active:"; systemctl status rust-server.service --no-pager -n 20 || true
    break
  fi
  sleep 10
done
address=$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{for (i = 1; i < NF; i++) if ($i == "src") print $(i + 1)}' | head -n 1)
if (( ready == 1 )); then
  stage "ready"
  echo "READY: in Rust press F1 and enter: client.connect ${address:-<server-ip>}:$GAME_PORT"
else
  stage "installed, but the game port is not listening yet"
  tail -n 30 "$SRV/logs/server.log" 2>/dev/null || true
  echo "NOT READY after waiting; run rust-status for details"
fi
echo "BOOTSTRAP DONE"
