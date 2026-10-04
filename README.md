# PC survival / FPS / Skate

![Rust x MW2 x Skate 3 banner](assets/rust-mw2-skate-banner.png)

We bouwen een zelfstandige **pc-survivalgame in Rust en Bevy**, met survival
en bases geïnspireerd door Rust, gunplay en operators geïnspireerd door MW2,
en skateboarden geïnspireerd door Skate 3. De wereld en gamecontent maken we
zelf: modellen, materialen, animaties en geluiden.

Alle tekst in het spel is Engels en blijft Engels: HUD, inventory, editor,
bedieningshints en meldingen. Dit geldt ook voor nieuwe bijdragen.

Het eindproduct moet zonder originele gamebestanden starten. Het gebruikt
publieke engine-/rewritecode en eigen content. De volledige featurelijst en
het onderscheid tussen aanwezige code, controles en speelbaarheid staan in
[TODO.md](TODO.md).

**Status: onafgewerkte ontwikkelbroncode. Er is nog geen afgewerkt speelbaar
product of releasebinary.** De repository dient als broncodeoverdracht en
voortgangsoverzicht. Wij verifiëren de implementatie zelf voordat we een
speelbare release leveren.

## Bijdragen

Vrijwilligers zijn welkom. Begin met de documentatie en een afgebakend issue,
werk op een eigen branch en lever een gerichte PR met controles en bijgewerkte
docs. De Engelstalige [CONTRIBUTING.md](CONTRIBUTING.md) beschrijft de werkwijze.

## Huidige implementatie

- Rust/Bevy-enginebasis met IW4L en de publieke MW2/Skate-integratie uit
  [2010-rust-rewrite-mashup](https://github.com/chasmlol/2010-rust-rewrite-mashup).
- Bouwkern: funderingen, vloeren, muren, deuren, eigendom, materialen,
  kosten/upgrades, ondersteuning/instorting en gevalideerde opslag.
- Autoritatieve kogel-/mêleeschade, loopcollisie en bouwstaat in wereldsnapshots.
- Lokale hostbouwbediening voor toetsenbord/muis en Xbox-controller op pc.
- Eigen procedurele hout-, steen- en metaalmaterialen zonder texturedownloads.
- Optionele SDK-v9-mapinspectietool; die is geen speelbare wereldimport.
- Zelfstandige sessie met eigen ingebedde gamescripts, een klein eigen eiland,
  first-person bewegen, schieten, schade, dood en respawn.
- Inventory met 24 slots, eindige startvoorraad, bandage-/munitierecepten,
  voedsel/water en honger/dorst; 30 eindige oogstbare materiaal-/voedselnodes.
- Eigen ramps, quarterpipes, rails, trappen, platforms en funboxes, met plaatsing,
  rotatie, verwijderen, undo/redo en gevalideerde sessie-opslag. Groen/rode previews
  tonen plaatsingsregels; bouwpreviews tonen de werkelijke materiaalkosten.
- Skatecontroller met pushen, sturen, remmen, ollies, spins/flips, landingspunten,
  bails en collisie met hetzelfde terrein en geplaatste ramps.
- Vier eigen 3D-modellen en zeven procedureel gemaakte CC0-geluiden, met
  reproduceerbare generators onder `scripts/`.

De eerdere sessiebootstrap is opgelost met onze eigen regels, zonder originele
gamescripts. Headless controles doorlopen nu bewegen, munitieverbruik, een NPC
uitschakelen, editorcollisie/undo/opslag, crafting, healing, harvesting,
dood/respawn en de overgang tussen skaten en lopen. De native frontend bevat
de bijbehorende bediening en feedback, inclusief inventory, gathering,
skatecamera, controllerbindings en geluidscues. De uitgebreide compilercontrole
en geoptimaliseerde executablebuild slagen. In een echt Bevy-venster op een
virtueel Linux-scherm zijn de eigen modellen, inventory/crafting, munitieverbruik,
schade, herladen, ADS en de skatecamera met ollie/landingspunten uitgevoerd.
Ook rampplaatsing/verwijderen/undo/redo, scene save/load, foundationkosten en
eindige tree-harvesting zijn via de native bediening uitgevoerd. Inventory-,
pauze- en focusovergangen behouden het camerabeeld. Native dood/respawn en
opnieuw bewegen/schieten zijn met een tijdelijke schadefixture geverifieerd;
die fixture is uit de productcode verwijderd.
Een volledige wereld, de overige survivalsystemen en multiplayerloops moeten
nog worden afgemaakt.
[TODO.md](TODO.md) scheidt implementatie, headless bewijs, grafische verificatie
en releasegereedheid.

## Bouwen en starten

Installeer Rust stable en de [systeemdependencies](docs/BUILD.md). Voor
compilercontrole vanuit de repository:

```bash
cargo check -p launcher -p rust_maps
```

Start de ontwikkelversie vanuit de repository:

```bash
cargo run -p launcher --profile play --locked -- game
```

De launcher kiest standaard hetzelfde zelfstandige native startpad. De eigen
content staat onder `assets/authored/` en wordt vanuit de repository gevonden.
F1 toont de bediening; Tab opent inventory, B bouwen, E de objecteditor en V skaten.
Richt in bouwmodus op een eigen bouwstuk: T repareert, Z upgrade naar Stone en
X naar Metal. Xbox Y repareert; D-pad Omhoog/Omlaag upgrade naar Stone/Metal,
Links/Rechts kiest een bouwstuk en LB/RB draait de plaatsingsas. B sluit de
bouwmodus. De bestaande Session-regels bepalen bereik, eigendom, gezondheid,
kosten en fouten. De melding "Place cost" hoort bij het plaatsingsvoorbeeld;
er is geen afzonderlijke reparatie-/upgradetargetpreview of kostenpreview
en geen sloopbediening.
Na een geslaagde F9-load is de bouwmodus uit; druk opnieuw B voor bouwacties.
In de inventory kiezen 1–9 of PgUp/PgDn een recept; C craft en R onderzoekt de
geselecteerde blauwdruk. O trekt de geselecteerde kleding aan, P trekt de
gedragen kleding uit en T repareert het geselecteerde gereedschap. N begint
recycling van de gekozen hoeveelheid; Enter bevestigt en Backspace annuleert. Xbox Back onderzoekt een
blauwdruk; Xbox Y/B bevestigen/annuleren een recyclingactie. Kledingbeheer en
reparatie/recycling starten voorlopig met het toetsenbord.
I gebruikt het geselecteerde item in de inventory (Xbox A doet hetzelfde).
Craft een hengel via recept 7; richt in loopmodus op bereikbaar water en druk
L om te vissen. L haalt een actieve lijn weer binnen. Xbox D-pad Rechts gebruikt
dezelfde visactie. Bij een kampvuur binnen bereik start G het bakken van één
rauwe vis voor 5 hout; na 15 simulatieseconden verzamelt G de gebakken vis. Xbox D-pad Links gebruikt
dezelfde kampvuuractie. Selecteer de gebakken vis in de inventory en gebruik I.
Richt op bereikbaar materiaal voor de verzamelhint; F oogst. F5/F9 bewaren/laden
de lokale sessie: scene, spelerpositie/kijkrichting, gezondheid, ammo en
gemonteerde skatestaat. Lopen/skaten, kijkrichting en ammo na laden zijn native
gecontroleerd. Dode saves herstellen ook positie, kijkrichting en snelheid;
de correctie uit [PR #13](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/13)
is onafhankelijk gecontroleerd voor dood→dood en levend→dood.
NPC's, wachtende scriptacties en editor-undo/redo worden niet opgeslagen.
Dit startpad is op Linux geverifieerd; Windows/macOS-builds zijn nog niet
geverifieerd. Er is nog geen kant-en-klare release-executable.
De bestaande [RUN.md](docs/RUN.md) en [SKATE.md](docs/SKATE.md) beschrijven de
upstreamgame-importmodi; hun releases en assetvereisten zijn niet de levering
of vereisten van de zelfstandige game die we hier bouwen.

## Verificatie en herkomst

Tijdelijke probes controleren bouwen, schade, mapvalidatie en de geïntegreerde
lokale sessie. Dezelfde terreinmesh wordt voor rendering en collisie gebruikt;
terreintraces, editorplaatsing en save/load zijn headless uitgevoerd. Inventory,
vitals, gathering en skatebeweging hebben daarnaast afzonderlijke controles
voor grenzen, ongeldige invoer en collisie. De eigen modellen en WAV-bestanden
zijn op bestandsstructuur en hashes gecontroleerd; audio regenereren levert
dezelfde bytes. De materiaalshader is met Naga gevalideerd. De eerdere
launcher/mapreader-compilercontrole en de uitgebreide workspacecontrole slagen.
GPU-weergave en eerder gecontroleerde keyboard/muisflows zijn op Mesa-software-rendering
uitgevoerd. De vis-/kampvuurflow en geselecteerd gebruik via I zijn gecontroleerd met
40 echte broncodecontroles, 2 aascompatibiliteitscontroles, 6 scene-updatecontroles
en 114 keyboard/Mesa-controles
(89 voor de volledige route, 25 voor foutgevallen);
[TODO.md](TODO.md) houdt de scopes afzonderlijk bij. Bouwreparatie en Stone/Metal-upgrades
zijn afzonderlijk gecontroleerd met 55 echte broncodecontroles, inclusief oude
formaat-3-saves zonder regentonveld, en 116 keyboard/Mesa-controles
(60 voor de route, 56 voor foutgevallen): proportionele reparatiekosten,
grade/health/materiaalwisseling, geweigerde acties zonder mutatie en F5/F9.
De broncodecontrole gebruikt synthetische controllerinvoer; grafische controles
gebruiken toetsenbord/muis. Dit is geen hardwareprestatiemeting.
Niet alle native flows zijn geverifieerd; hoorbaar geluid, Xbox-controllerhardware
en multiplayer blijven open.

Upstreamdocumentatie staat in [UPSTREAM.md](docs/UPSTREAM.md); credits en
licenties staan in [NOTICE](NOTICE) en [LICENSE](LICENSE). Oorspronkelijke
gamecontent wordt niet meegeleverd. De afzonderlijke licentiescope van de
meegeïmporteerde SK8/MinecraftOSS-modules en herkomst van oudere numerieke data
blijven onderdeel van de review. De geïmporteerde engine is geen bewijs van
een volledig nieuw geschreven of onafhankelijk geverifieerde clean-room engine.
