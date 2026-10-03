# TODO — pc-survival met FPS en skaten

Doel: een zelfstandige pc-game in Rust/Bevy, met survival en bouwen geïnspireerd
door Rust, FPS-gunplay en operators geïnspireerd door MW2, en skateboarden
geïnspireerd door Skate 3. Werelden, modellen, materialen, animaties en geluiden
maken we zelf. Originele gamebestanden zijn geen vereiste voor het eindproduct.

**Status op 3 oktober 2026: onafgewerkte ontwikkelbroncode. Geen afgewerkt
speelbaar product en geen releasebinary.** De repository bevat de implementatie
en dient als broncodeoverdracht, met onderstaande verificatiegrenzen.

We houden vier stadia apart: **code aanwezig**, **headless geverifieerd**,
**grafisch geverifieerd** en **releasegereed**. Headless controles voeren de
echte backend zonder venster uit. Ze bewijzen geen camerabeeld, bediening of
geluid. `[x]` hieronder geldt uitsluitend voor de expliciet genoemde controle;
de productroadmap blijft open totdat de volledige gebruikersflow is geverifieerd.

## Wat aantoonbaar aanwezig is

| Onderdeel | Code aanwezig | Headless/codebewijs | Grafisch geverifieerd | Releasegereed |
|---|---|---|---|---|
| Bouwkern: kosten, eigendom, upgrades, deuren, instorting, opslag | Ja | 8 bouwscenario's geslaagd | Nee | Nee |
| Kogel-/mêleeschade aan bouwstukken, voorspelling zonder schade | Ja | 5 simulatieprobes geslaagd | Nee | Nee |
| SDK-v9-mapreader | Ja, optionele inspectietool | 8 validatieprobes en synthetische inspectie geslaagd | Niet van toepassing | Geen gamewereldimport |
| Bouwstukken in snapshots en loopcollisie | Ja | Compilercontrole en lokale collisionprobes | Nee; netwerkflow open | Nee |
| Procedurele bouwmaterialen | Ja | WGSL/Naga-validatie geslaagd | Nee | Nee |
| Zelfstandige sessie met eigen gamescripts | Ja | Start, bewegen, schieten, NPC-kill, dood en respawn geslaagd | FPS, ammo, NPC-schade/kill, herladen en ADS uitgevoerd; dood/respawn open | Nee |
| Klein eigen eiland en zes skateobjecttypen | Ja | 15 terreintraces, plaatsing/rejectie, undo/redo en save/load geslaagd | Terrein zichtbaar; editorflow in uitvoering | Nee |
| Inventory, twee recepten, consumables, honger/dorst | Ja | Grenzen/persistentie plus geïntegreerde resourcekosten, healing en ammo geslaagd | Inventorypauze, beide craftkosten en ammo-transfer uitgevoerd | Nee |
| 30 eindige oogstnodes | Ja | Generatie, reach/occlusie, depletion en geïntegreerde tree-harvest/opslag geslaagd | Nodes zichtbaar; interactie open | Nee |
| Skatecontroller en loop/skate-overgang | Ja | Push/steer/ollie/tricks/bail/collisie en gemonteerde push/handoff geslaagd | Camera/mount, push, ollie/landingsscore en afstappen uitgevoerd | Nee |
| Eigen operator, carbine, board en timber-model | Ja | GLB-structuur, scene-aantallen en hashes gecontroleerd | Operator/carbine/board zichtbaar; volledige riganimatie open | Nee |
| Zeven eigen korte CC0-WAV-cues | Ja | PCM/manifestcontrole en identieke regeneratie geslaagd | Playback nog open | Nee |
| Native controller, ADS/recoil, inventory-/gather-/skatefeedback en audiohooks | Ja | Workspacecontrole en geoptimaliseerde build slagen | Deel van keyboard/muisflows uitgevoerd; audio/hardware open | Nee |
| Engelse in-game tekst | Ja, vaste projectregel | HUD, inventory, controls, feedback en backenderrors nagekeken | HUD/inventory/editor/pauze in Engelse build uitgevoerd | Nee |

- [x] Bestaande bouwscenario's, schadeprobes en mapvalidatie uitgevoerd.
- [x] Procedurele materiaalshader geparseerd en gevalideerd.
- [x] Gedownloade bouwtextures en downloadscript uit productpad gehaald.
- [x] Eerste native frontend, launcher en mapreader compileren succesvol.
- [x] Uitgebreide native frontend met controller/UI/audio en scene-reflectie compileren.
- [x] Eigen regels laden/starten zonder originele gamescripts; authority blijft actief.
- [x] Lokale sessie doorloopt bewegen, schieten en een NPC uitschakelen.
- [x] Eigen terrein en editorplaatsing/collisie/undo/redo/save/load headless uitgevoerd.
- [x] Craftkosten, bandagegebruik/healing, ammo-refill en finite harvesting geïntegreerd gecontroleerd.
- [x] Dode spelers kunnen niet craften/healen/gatheren; respawn en schade na respawn gecontroleerd.
- [x] Gemount skaten, pushen en terug naar lopen geïntegreerd gecontroleerd.
- [x] Eigen WAV-pack gecontroleerd en byte voor byte geregenereerd.
- [x] Geoptimaliseerde native executable bouwen en eigen modellen/terrein in het venster controleren.
- [x] Inventorypauze, crafting/resourcekosten/ammo, schieten/schade/herladen/ADS grafisch uitvoeren.
- [x] Skatecamera, pushen, ollie en landing met score via keyboard uitvoeren.
- [x] Alle in-game teksten naar Engels omzetten; Engels als vaste bijdragersregel vastleggen.
- [ ] Volledige native editor-/bouw-/gather-/respawnflow en Engelse meldingen verifiëren.

## Actuele verificatiefase

De sessiebootstrap werkt nu met eigen ingebedde authority-gamescripts. De
geïntegreerde backendprobes, uitgebreide workspacecontrole en geoptimaliseerde
executablebuild slagen. Het eigen terrein en de modellen verschijnen in een echt
Bevy-venster via Mesa-software-rendering. Inventory/crafting, schieten/schade,
herladen/ADS en skatecamera/push/ollie/landingspunten zijn via keyboard/muis uitgevoerd.
De editor en overige flows worden verder gecontroleerd. Alle spelteksten zijn
Engels; nieuwe bijdragen moeten die taal behouden.
Xbox-controllerbindings betekenen nog geen hardwareverificatie. Native audio
playback is nog niet bewezen. Geen van deze stappen is een releaseverklaring.

De overdracht voor Claude staat in [CLAUDE_HANDOFF.md](CLAUDE_HANDOFF.md).

## Eerst: zelfstandig pc-product

- [ ] Native Bevy-startpad zonder MW2-, Rust- of Skate-gamebestanden.
- [ ] Eigen startmenu, instellingen, pauze, laadscherm, respawn en afsluiten.
- [ ] Eigen operator, wapens, skateboard, bouwstukken en wereldmodellen.
- [ ] Eigen rigging, animaties, materiaalvarianten, VFX en geluiden.
- [ ] Geïntegreerde speelwereld met spawn, interacties en duidelijke feedback.
- [ ] Toetsenbord/muis en Xbox-controller, remapping, sensitiviteit en deadzones.
- [ ] Reproduceerbare Windows-build; Linux/macOS-builds volgens gekozen support.
- [ ] Verpakte executable met noodzakelijke eigen content en startinstructies.

## Survival: bewegen, verzamelen, inventory en crafting

- [ ] Lopen, sprinten, springen, hurken, zwemmen, vallen en ondergrondgedrag.
- [ ] Itemcatalogus, inventory/hotbar, stapelen, splitsen, droppen en containers.
- [ ] Bomen, stenen, ertsen, planten en oogstbaar materiaal in de wereld.
- [ ] Gereedschap, gathering-opbrengsten, slijtage, reparaties en recycling.
- [ ] Craftingrecepten, wachtrij, werkbanken, onderzoek en blueprintprogressie.
- [ ] Gezondheid, schade, bloeden, honger, dorst, temperatuur en straling.
- [ ] Voedsel, koken, medicijnen, kleding, pantser en bescherming.
- [ ] Dood, corpses/lootbags, sleeping bags/bedden en respawnregels.

## Bases, eigendom en raiden

- [ ] Bestaande bouwkern koppelen aan echte inventory en gathering.
- [ ] Bouwpreview, snapping, rotatie, meerdere vormvarianten en terreinplaatsing.
- [ ] Volledige ondersteuning/stabiliteit met zichtbare feedback.
- [ ] Bouwkasten/tool cupboards, autorisatie, bouwrechten en bouwblokkering.
- [ ] Hout/steen/metaal/hoogwaardig metaal, upgrades, reparatie en upkeep/decay.
- [ ] Deuren, luiken, ramen, poorten, trappen, sloten, sleutels en codes.
- [ ] Opslagkisten, ovens, werkbanken, deployables en interieur.
- [ ] Raidregels, explosieven, projectielschade, zwakke zijden en puin.
- [ ] Vallen, verdediging, turrets, schadebalans en offline-basepersistentie.
- [ ] Bouwacties voor multiplayerclients en controllerbediening voor alle acties.

## Elektriciteit, industrie, landbouw en vloeistoffen

- [ ] Generatoren, zonnepanelen, wind, batterijen en stroomcapaciteit.
- [ ] Kabels, schakelaars, splitters, logica, sensoren en elektrische verbruikers.
- [ ] Industriële conveyors, filters, automatische verwerking en opslagroutes.
- [ ] Zaden, groei, grond, licht, water, mest, kruisingen en oogsten.
- [ ] Pompen, tanks, slangen, waterkwaliteit, irrigatie en vloeistofstromen.
- [ ] Vissen, aas, hengels, vangsten, visverwerking en voedselketen.

## Wereld, loot en PvE

- [ ] Eigen terrein, biomen, rotsen, begroeiing, water en onderwatergebieden.
- [ ] Wegen, tunnels, monumenten, gebouwen en terrein-/interieurcollisie.
- [ ] Lootcontainers, loot-tabellen, tiers, respawntimers en locked crates.
- [ ] Dag/nacht, weer, seizoens-/temperatuurinvloeden, verlichting en ambiance.
- [ ] Dieren en NPC's met navigatie, waarneming, gevechten en loot.
- [ ] Wereldactiviteiten, airdrops, patrouilles en monument-events.
- [ ] Veilige zones, handel, vending, economie en NPC-missies.
- [ ] Map, kompas, markers, spawnregels en reproduceerbare world seeds.

## Eigen map en skateobjecteditor

- [ ] Eigen gegenereerd survivalterrein met hoogte, biomen en vertexmaterialen.
- [ ] Dezelfde terreingeometrie gebruiken voor rendering en loop-/skatecollisie.
- [ ] Editor aan/uit met E; eigen ramps, quarterpipes, rails, trappen, platforms en funboxes.
- [ ] Objecttype kiezen, op gericht terrein plaatsen en per 15° draaien.
- [ ] Plaatsingsghost met geldige/ongeldige feedback en collisiecontrole.
- [ ] Bestaande objecten selecteren, verplaatsen, roteren en laten snappen.
- [ ] Raster-/hoogte-/hoek-snapping en kopiëren/dupliceren van objecten.
- [ ] Objecten verwijderen en wijzigingen ongedaan/opnieuw met Ctrl-Z/Ctrl-Y.
- [ ] Terrein/objectstaat samen opslaan/laden, met begrensde validatie en versies.
- [ ] Ramps/quarterpipes volgen met skatefysica en grinds op geplaatste rails.
- [ ] Speler/objectoverlap, bullet-/vehicle-/skatecollisie en netwerkbouwrechten.
- [ ] Native editorflow en save-load/undo-redo zelf grafisch verifiëren.

## Voertuigen en vervoer

- [ ] Paarden of ander landtransport, instappen, besturing en inventaris.
- [ ] Auto's/modulaire voertuigen, onderdelen, brandstof, reparatie en schade.
- [ ] Boten, drijfvermogen, motoren, opslag en waterschade.
- [ ] Lucht-/railtransport voor de gekozen spelwereld.
- [ ] Passagiers, botsingen, voertuigpersistentie en multiplayerreplicatie.

## FPS-gunplay en operators

- [ ] Eigen operators, first-/third-person rigs, outfits en equipmentweergave.
- [ ] Wapenmodelcatalogus met eigen meshes, materialen en geluiden.
- [ ] FPS-movementfeel, ADS, hipfire, sprintgedrag en camerabeweging.
- [ ] Recoilpatronen, terugkeer, sway, spread, ademcontrole en handling.
- [ ] Munitietypen, magazijnen, reloads, fire modes en wapenstaat.
- [ ] Attachments, sights, scopes, suppressors en gebalanceerde modifiers.
- [ ] Schot-, reload-, equip-, inspectie- en mêleeanimaties plus VFX/audio.
- [ ] Trefferzones, pantser, penetratie, projectielen en lagcompensatie.
- [ ] Granaten/explosieven, gadgets en aansluiting op survivalcrafting/raiden.
- [ ] Consistente gunplay voor speler, NPC's en multiplayerclients.

## Skateboarden

- [ ] Bestaande skatekern voeden met eigen rig, board en animaties.
- [ ] Op-/afstappen, rollen, pushen, remmen, sturen en camerawissels.
- [ ] Ollies, flips, grabs, manuals, landingregels en tricks combineren.
- [ ] Grinds, slides, raildetectie, transferregels en ramps/transitions.
- [ ] Bails, ragdolls/herstel, valimpact en survivalgezondheid.
- [ ] Collisie met terrein, monumenten, nieuwe bases en andere spelers.
- [ ] Netwerkreplicatie van skater, board, trickstaat en herstel.
- [ ] Controller-flickbediening, kalibratie en toetsenbordalternatief.
- [ ] Replay, camera's en montage/clip-export na stabiele gameplay.

## Multiplayer, opslag en beheer

- [ ] Autoritatieve serverflow voor alle survival-, FPS- en skateacties.
- [ ] Join/leave, spawning, wereldsync, ownership en reconnect.
- [ ] Inventory/crafting/gathering/bouwverzoeken met servervalidatie.
- [ ] Persistentie voor spelers, bases, loot, voertuigen, stroom en landbouw.
- [ ] Save/load-versies, crashherstel, backups, world seeds en wipebeheer.
- [ ] Teams, permissions, chat en voice voor de gekozen productversie.
- [ ] Serverbrowser/direct join, dedicated server, instellingen en adminacties.
- [ ] Bescherming tegen ongeldige pakketten, resource-abuse en cheats.
- [ ] Twee echte clients, latency/packet loss en langdurige serverruns verifiëren.

## Afronding en levering

- [ ] Geïntegreerde loops verifiëren: verzamelen → craften → bouwen → verdedigen.
- [ ] Gevechten, dood/respawn, skaten, opslag en multiplayer samen doorlopen.
- [ ] Performance, streaming, geheugen, graphicskwaliteit en toegankelijkheid.
- [ ] Heldere in-game hints, foutmeldingen, controllerprompts en settingsopslag.
- [ ] Licenties/herkomst van gebruikte code en eigen content controleren.
- [ ] Afzonderlijke SK8/MinecraftOSS-licentiescope en herkomst van oudere numerieke data afhandelen.
- [ ] Afgewerkt product reviewbaar maken en een geverifieerde speelbare release leveren.

Deze lijst beschrijft de ambitie en ontbrekende integratie. Ze beweert geen
volledige Rust-pariteit of percentage voltooiing; extra subsystemen worden
concreet toegevoegd zodra de gekozen spelregels en implementatie daarom vragen.
