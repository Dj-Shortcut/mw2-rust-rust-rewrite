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
| Zelfstandige sessie met eigen gamescripts | Ja | Start, bewegen, schieten, NPC-kill, dood en respawn geslaagd | FPS/ammo/ADS en dood/respawn via tijdelijke schadefixture uitgevoerd; bewegen/schieten na respawn werkt | Nee |
| Klein eigen eiland en zes skateobjecttypen | Ja | 15 terreintraces, plaatsing/rejectie, undo/redo en save/load geslaagd | Ramp plaatsen/verwijderen, undo/redo, scene save/load en railrotatie uitgevoerd | Nee |
| Read-only bouw-/objectpreview met groen/rood voorbeeld | Ja | Exacte preview/commit-pariteit, kosten en geen mutatie gecontroleerd | Groen/rood, foundationplaatsing met 200 hout en bezette-plekfeedback uitgevoerd | Nee |
| Inventory, twee recepten, consumables, honger/dorst | Ja | Grenzen/persistentie, resourcekosten, healing en ammo; native UI-state en softwarebindings gecontroleerd | Keyboard: pauze, craftkosten, ammo, splitsen, merge/swap en bevestigd vernietigen; F5/F9 behoudt aantallen/volgorde | Nee |
| Lootbags bij dood en native oprapen | Ja | 39 Session/helper/ECS-controles en 52 Bevy-input-/dispatchcontroles geslaagd | Keyboard/Mesa: respawn, teruglopen, volledig/gedeeltelijk oprapen, volle inventory, modeblokkering en F5/F9 uitgevoerd | Nee |
| 30 eindige oogstnodes | Ja | Generatie, reach/occlusie, depletion en pure target/harvest-pariteit geslaagd | Tree-hint, 12 oogsten, verdwijnen bij uitputting en voorraad/depletion na laden uitgevoerd | Nee |
| Skatecontroller en loop/skate-overgang | Ja | Push/steer/ollie/tricks/bail/collisie en gemonteerde push/handoff geslaagd | Camera/mount, push, ollie/landingsscore en afstappen uitgevoerd | Nee |
| Eerste authored railgrinds en Engelse grind-HUD | Ja | 26 live railscenario's, 28 landings-/bailcontroles en 14 score-/savegroepen geslaagd | F9 hervatten, keyboard Space→rail→landing en afstappen met scorebehoud uitgevoerd via Mesa en gecontroleerde saves | Nee |
| Eigen operator, carbine, board en timber-model | Ja | GLB-structuur, scene-aantallen en hashes gecontroleerd | Operator/carbine/board zichtbaar; volledige riganimatie open | Nee |
| Zeven eigen korte CC0-WAV-cues | Ja | PCM/manifestcontrole en identieke regeneratie geslaagd | Playback nog open | Nee |
| Native controller, ADS/recoil, inventory-/gather-/skatefeedback en audiohooks | Ja | Workspacecontrole en geoptimaliseerde build slagen | Deel van keyboard/muisflows uitgevoerd; audio/hardware open | Nee |
| Lokale sessie-save/load (formaat 3, migratie van formaat 2) | Ja | Backendprobes, cameradelta en dode transforms na PR #13 onafhankelijk gecontroleerd | F5/F9: lopen/gemount, positie/kijkrichting/ammo, ongeldige load en verder spelen gecontroleerd | Nee |
| Engelse in-game tekst | Ja, vaste projectregel | HUD, inventory, controls, feedback en backenderrors nagekeken | HUD/inventory/editor/pauze in Engelse build uitgevoerd | Nee |

- [x] Bestaande bouwscenario's, schadeprobes en mapvalidatie uitgevoerd.
- [x] Bouwreparatie headless geverifieerd (#39): `BuildingWorld::repair` herstelt tot het grademaximum tegen een proportioneel, naar boven afgerond aandeel van de gradekosten; vol/missend/vreemd/arm-afwijzing is atomair; save round-trip behoudt gerepareerde health; demolish/instorting blijven werken. Alleen `rust_building`-backend via `context/repair-probe`; Session-/native-bediening, upkeep/decay en grafische verificatie open.
- [x] Reparatie vanuit het vizier headless geverifieerd (#42): `Session::repair_from_view` repareert het geraakte eigen bouwwerk tegen proportionele kosten; mis/vreemd/vol/dood-afwijzing is atomair zonder lading of genezing; save/load behoudt gerepareerde health en balans. Alleen headless via `context/repair-view-probe`; native-binding en grafische verificatie open.
- [x] Upgrade/sloop vanuit het vizier headless geverifieerd (#46): `Session::upgrade_from_view`/`demolish_from_view` delen de vizier-trace met gelijkvlakvoorrang; upgrade rekent gradekosten en reset health, sloop retourneert verwijderde IDs inclusief instorting; mis/vreemd/gelijk/arm/dood-afwijzing is atomair; save/load behoudt grades en balans. Alleen headless via `context/upgrade-demolish-probe`; native-binding en grafische verificatie open.
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
- [x] Inventorystapels splitsen, samenvoegen/wisselen en weggooien headless via `Session` gecontroleerd, inclusief geweigerde acties zonder wijziging, volle inventory, dode speler en save/load ([#31](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/31)). Handmatige wereld-drops en containers blijven open; vernietigen maakt geen lootbag.
- [x] Native inventorystapelbediening ([#41](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/41)): keyboardselectie/hoeveelheid, splitsen, volledig verplaatsen, merge tot stacklimiet en wisselen, vernietigen met aparte bevestiging en annuleren. Via Mesa op 1280×720 gecontroleerd, inclusief volle/lege/dode gevallen, close/focus/pauze, leesbare Engelse fouten, F5/F9 met aantallen/volgorde en weigeren van ongeldige load. Open inventory blokkeert lopen/schieten/kijkinvoer. 17 echte Session/UI-probegroepen en 133 Bevy-inputcontroles slagen, inclusief oude v3-saves en verbandgebruik bij volle health met/zonder bloeden; die verbandflow is ook native uitgevoerd; Xbox-hardware, muisdrag/drop en handmatige wereld-drops blijven open.
- [x] Lootbag bij dood headless via `Session` gecontroleerd: inventory valt in een tas op de grond, na respawn teruglopen en (gedeeltelijk) oprapen, maximaal 16 tassen, dode/oude saves en geweigerde gemanipuleerde saves ([#36](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/36)).
- [x] Native lootbags en herstel ([#56](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/56)): eigen procedurele tassen op de death-droppositie; F / Xbox Y haalt spullen uit de dichtstbijzijnde tas binnen bereik vóór verzamelen, met Engelse hints en feedback. 39 echte Session/helper/ECS-controles en 52 Bevy-input-/dispatchcontroles slagen, inclusief gedeeltelijk herstel, scene-opruiming, baglimiet en geweigerde acties. Via keyboard/Mesa uitgevoerd: respawn, teruglopen, oprapen, volle inventory, vijf items ophalen en 25 later terughalen, modeblokkering en F5/F9 met achtergebleven inhoud. Geoptimaliseerde native build slaagt; fysieke Xbox-controller, hoorbare audio en releasegereedheid blijven open. Handmatig vernietigen maakt geen wereld-drop.
- [x] Bloeden headless via `Session` gecontroleerd: een externe treffer van 15+ schade start bloeden (pool = halve schade, max 40, ~1 HP/s), een verband stopt het ook bij volle health, respawn wist het, saves bewaren het en gemanipuleerde waarden worden geweigerd ([#43](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/43)). Nog geen HUD-weergave.
- [x] Lege grondstofnodes groeien headless na 300 s terug naar volle voorraad; een levende speler, editorobject of bouwdeel op de plek blokkeert (opnieuw proberen na 5 s), saves bewaren de timer en gemanipuleerde timers worden geweigerd ([#50](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/50)).
- [x] Medische spuit headless via `Session` gecontroleerd: craften (15 hout, 20 metaal, stapel 5), direct 15 HP plus 20 HP over 10 s, stopt bloeden, pool max 40, respawn wist hem, saves bewaren hem en gemanipuleerde waarden worden geweigerd ([#54](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/54)). Native toetsen voor recept 3 en gebruik ontbreken nog.
- [x] Recyclen headless via `Session::recycle_stack` gecontroleerd: gecrafte items leveren de helft van hun craftkosten op (verband 10 hout, 30 kogels 5 steen en 7 metaal, spuit 7 hout en 10 metaal), voedsel en water niet, munitie per 30, geweigerd bij dood of volle opslag en zonder wijziging bij fouten ([#58](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/58)). Nog geen native bediening.
- [x] Craftingwachtrij headless via `Session::queue_craft`/`cancel_craft` gecontroleerd: kosten worden direct betaald, items komen na hun crafttijd (verband 3 s, 30 kogels 5 s, spuit 10 s), alleen de eerste taak loopt, maximaal 8 taken, annuleren geeft alles terug, een volle inventory laat de klare taak wachten, dood geeft de kosten terug (tot de opslaglimiet), saves bewaren de wachtrij en gemanipuleerde wachtrijen worden geweigerd ([#60](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/60)). Het directe `craft` blijft voor de native toetsen; native/HUD-weergave van de wachtrij ontbreekt nog.
- [x] Stenen bijl headless via `Session` gecontroleerd: craften (100 hout, 50 steen, 8 s), dubbele opbrengst op bomen, steen en erts (bessen en water niet), één slijtage per slag en breken na 50 slagen; slijtage blijft bewaard bij weggooien, lootbags en saves, recyclen schaalt met de staat en gemanipuleerde slijtage wordt geweigerd ([#63](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/63)). Geen native bediening of uitrusten.
- [x] Gereedschap repareren headless via `Session::repair_tool` gecontroleerd: de slijtage gaat naar 0 voor de helft van de craftkosten naar rato van de slijtage, naar boven afgerond (bijl met 25 slijtage: 25 hout en 13 steen), en wordt zonder wijziging geweigerd bij dood, te weinig grondstoffen, een lege slot, een niet-gereedschap of een onversleten bijl ([#68](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/68)). Geen native bediening.
- [x] Stenen pikhouweel headless gecontroleerd: zelfde recept en slijtage als de bijl. De bijl verdubbelt nu alleen bomen en het pikhouweel steen en erts; verkeerd gereedschap helpt niet en slijt niet, beide breken na 50 slagen met een eigen melding ([#70](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/70)). Geen native bediening.
- [x] Blueprints headless gecontroleerd: injectiespuit en pikhouweel moeten eerst onderzocht worden voor het dubbele van hun receptkosten. Craften en in de wachtrij zetten weigert zonder blueprint; blueprints blijven na de dood en staan in de save, oude saves laden met alleen de basisrecepten ([#72](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/72)). Geen native bediening; de HUD toont vergrendelde recepten nog.
- [x] Native gereedschapsfeedback ([#67](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/67)): de brekende oogst behoudt opbrengst/restvoorraad en toont de juiste Engelse bijl- of pikhouweelmelding; de volgende oogst herhaalt die niet. Gereedschap wisselen toont "Stacks swapped", gewone stapels houden "Stacks merged". 56 echte Session/InventoryUi/native-input-/dispatchcontroles en 109 keyboard/Mesa-controles slagen: verkeerd gereedschap slijt niet, slijtage/itemvolgorde blijven bij wisselen en F5/F9 behouden, gewone volledige/gedeeltelijke merges en volle-doelafwijzing behouden aantallen. Geoptimaliseerde native build en repositorychecks slagen. Geen nieuwe crafting-, uitrust-, onderzoek- of reparatiebediening; controllerhardware, hoorbare audio en releasegereedheid blijven open.
- [x] Dag/nachtklok en temperatuur headless gecontroleerd: een dag duurt 30 minuten, van 22 °C om 14:00 tot -4 °C om 02:00. Onder 5 °C daalt honger dubbel zo snel, onder 0 °C kost vriezen 0,05 HP/s met een melding; de klok loopt door na de dood en staat in de save ([#74](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/74)). Geen kleding, vuur of native weergave.
- [x] Gewatteerde jas headless gecontroleerd: recept 60 hout + 10 metaal, één kledingslot met aantrekken, wisselen en uittrekken; de jas maakt de gevoelstemperatuur 8 °C warmer zodat 02:00 niet meer vriest. Bij de dood gaat de jas in de lootbag en hij staat in de save ([#76](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/76)). Geen pantser of native bediening.
- [x] Lootkratten headless gecontroleerd: vier tonnen en twee militaire kratten op vaste plekken, met gewogen loottabellen (2 of 3 trekkingen) die deterministisch zijn per seed, krat en opening. Openen is alles-of-niets, kratten vullen na 300 of 600 s opnieuw, ook tijdens de dood, en de timers staan in de save ([#79](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/79)). Geen modellen, locked crates of native bediening.
- [x] Vissen headless gecontroleerd: hengel (60 hout + 5 metaal, gereedschap met slijtage, reparatie en recycling), uitwerpen op een waterknoop, na 6 s 60% kans op rauwe vis (+20 honger), deterministisch per seed, plek en teller. Wegwandelen, hengel kwijt of de dood haalt de lijn in; worp en teller staan in de save ([#83](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/83)). Geen aas, soorten, koken of native bediening.
- [x] Straling headless gecontroleerd: een zone van 450 eenheden rond beide militaire kratten laat straling 2 per seconde stijgen tot 100 en buiten de zone 0,5 per seconde dalen; boven 40 kost het 0,1 HP per seconde. Binnenkomen geeft één melding. Anti-stralingspillen (10 hout + 15 metaal, 3 s) halen 50 weg. Straling staat in de vitals van de save en verdwijnt bij respawn ([#85](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/85)). Geen beschermende kleding, HUD-balk of geluid.
- [x] Eigen WAV-pack gecontroleerd en byte voor byte geregenereerd.
- [x] Geoptimaliseerde native executable bouwen en eigen modellen/terrein in het venster controleren.
- [x] Inventorypauze, crafting/resourcekosten/ammo, schieten/schade/herladen/ADS grafisch uitvoeren.
- [x] Skatecamera, pushen, ollie en landing met score via keyboard uitvoeren.
- [x] Alle in-game teksten naar Engels omzetten; Engels als vaste bijdragersregel vastleggen.
- [x] F5/F9 native uitvoeren: positie/kijkrichting/ammo herstellen, lopen/gemount herstellen en ongeldige load weigeren zonder speelstaatverlies.
- [x] Native invoer synchroniseren met geladen kijkrichting en cameradelta; effectieve pitch begrenzen zodat volgende invoer de herstelde hoek behoudt.
- [x] F5 bij dood, F9 dood→levend/levend→dood, 75° kijkhoek na respawn, action gate en verder spelen via tijdelijke native fixture uitvoeren.
- [x] Dead-posecorrectie uit [PR #13](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/13) onafhankelijk controleren: positie, kijkrichting en snelheid bij dood→dood en levend→dood; ongeldige load blijft atomair. #14 is opgelost.
- [x] Native herhaald F9 met dode saves, Enter-respawn zonder yaw-sprong en daarna lopen/skaten/schieten controleren; geaccepteerde commandhoek behouden bij dode loads.
- [x] Pure previews verifiëren: alle zes objecttypen, geldige/ongeldige bouwkosten en geen wijzigingen vóór plaatsing.
- [x] Bouwpreview zonder gekloonde bouwwereld: gedeelde read-only validatie, collision-/foutpariteit en statebehoud gecontroleerd. Bij 4.095 bouwstukken vraagt de preview 288.040 bytes en 684 allocaties minder per query dan het behouden clone-pad; de gemeten tijd bewijst geen versnelling.
- [x] Ramp plaatsen/verwijderen, undo/redo, scene save/load en foundationkosten via native bediening uitvoeren.
- [x] Gerichte verzamelhint, finite tree-harvest/depletion en herstel van voorraad/depletion grafisch uitvoeren.
- [x] Ongewijzigd camerabeeld na inventory sluiten, pauze hervatten en focus terugkrijgen controleren.
- [x] Native dood/respawn en opnieuw bewegen/schieten via tijdelijke schadefixture uitvoeren; fixture verwijderen.
- [ ] Alle objectvarianten, deur-/upgrade-/sloopbediening en gekoppelde skatecollisie grafisch doorlopen.
- [x] Volledige sessie headless opslaan en laden: positie, snelheid, kijkrichting, health, levend/dood (GSC-lifecycle), wapen, clip/reserve, kills/deaths/score, gemounte skatestaat en -score, plus inventory, bouwresources, needs, depletion, gebouwen en editorobjecten.
- [x] Dode spelers laden met opgeslagen positie, kijkrichting en snelheid (dood→dood en levend→dood), gecontroleerd vóór commit; probe met twee doden op verschillende plekken, herhaald laden en respawn.
- [x] Rail grinds headless via `Session::advance`: vangen na een echte ollie (yaw 0/90/37, beide richtingen), afglijden, loslaten aan het eind, ollie-uit en remmen; grindpunten pas bij een veilige landing; bails, near misses, muur en verwijderen/undo vangen niets of vervallen; mid-grind save/load en oude saves gecontroleerd.
- [x] Ledge grinds headless via `Session::advance` (#27): funboxdeck (twee lange randen) en platform (vier randen) zijn grindbaar met dezelfde vang-, glij-, los- en scoreregels als de rail. Ollie vanaf de kicker op de funboxrand en vanaf de grond op de platformrand (yaw 0/37/90, beide randen, beide richtingen) grinden en landen met één beloning; over het deck rijden zonder ollie, een ollie midden op het deck en langs de zijkant rijden vangen niets. Mid-ledge save/load hervat op dezelfde rand; een ongeldige of verkeerde randindex weigert de hele load; rail-saves zonder randveld laden als rand 0.
- [x] Engelse native grind-HUD met snelheid, verdiende/pending punten en controls uitvoeren: F9 mid-grind → veilig landen met 34 punten → V afstappen behoudt 34; Space vanaf een gecontroleerde approach-save vangt de rail en landt met 32 punten. De startsnelheid van 220 is fixturedata; Xbox-hardware en overige skateobjecten blijven open.
- [ ] Volledige native editor-/bouw-/gather-/respawnflow en Engelse meldingen verifiëren.

## Actuele verificatiefase

De sessiebootstrap werkt nu met eigen ingebedde authority-gamescripts. De
geïntegreerde backendprobes, uitgebreide workspacecontrole en geoptimaliseerde
executablebuild slagen. Het eigen terrein en de modellen verschijnen in een echt
Bevy-venster via Mesa-software-rendering. Inventory/crafting, schieten/schade,
herladen/ADS en skatecamera/push/ollie/landingspunten zijn via keyboard/muis uitgevoerd.
Daarbij zijn nu rampplaatsing/verwijderen/undo/redo, scene save/load, foundationkosten,
groen/rode previews en eindige tree-harvesting met herstel na laden gecontroleerd.
Inventory-, pauze- en focusovergangen behouden hetzelfde camerabeeld. Een tijdelijke
schadefixture controleert native dood/respawn en opnieuw bewegen/schieten; die fixture
zit niet in de productcode. Alle spelteksten blijven Engels; F1 toont de bediening.
Claude's formaat-3 opslag is geïntegreerd met de native invoer. F5/F9 herstelt de
lopende/gemonteerde speler en behoudt positie, kijkrichting en ammo; een ongeldige
load laat de sessie bruikbaar. De dead-posecorrectie uit PR #13 is geïntegreerd
en onafhankelijk gecontroleerd; #14 is opgelost. Volledige persistente wereldstaat
en multiplayer blijven open.
De native schadefixture controleert ook opslag tijdens dood, laden tussen levend
en dood en een geladen kijkhoek van 75° na respawn. Geen fixturecode wordt geleverd.
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
- [ ] Hout/steen/metaal/hoogwaardig metaal, upgrades en upkeep/decay; reparatie-backend en vizierbediening aanwezig (#39/#42, headless), native-binding open.
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
- [x] Plaatsingsghost met geldige/ongeldige feedback en collisiecontrole.
- [ ] Bestaande objecten selecteren en laten snappen; verplaatsen en roteren vanuit het vizier aanwezig (#49, headless), native-binding open.
- [ ] Raster-/hoogte-/hoek-snapping en kopiëren/dupliceren van objecten.
- [x] Objecten verwijderen en wijzigingen ongedaan/opnieuw met Ctrl-Z/Ctrl-Y.
- [x] Editor-props verplaatsen/roteren vanuit het vizier headless geverifieerd (#49): `EditorState::relocate` met validatie en undo-checkpoint; `Session::move_prop_to_view`/`rotate_prop_from_view` met bestaande vizier-lookup, terreinbestemming zonder de prop zelf, speler-overlapafwijzing en dode-speler-blokkade; mislukte acties nemen geen undo-slot; rails behouden grind-ID op het verplaatste segment; save/load behoudt poses. Alleen headless via `context/prop-relocate-probe`; selectie-UI, snapping en native-binding open.
- [ ] Terrein/objectstaat samen opslaan/laden, met begrensde validatie en versies.
- [ ] Ramps/quarterpipes volgen met skatefysica en grinds op geplaatste rails. [Claude-taak #16](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/16) werkt de eerste railgrinds uit.
- [ ] Speler/objectoverlap, bullet-/vehicle-/skatecollisie en netwerkbouwrechten.
- [x] Native ramp-editorflow en scene save-load/undo-redo zelf grafisch verifiëren.

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
- [ ] Crashherstel, backups, world seeds en wipebeheer. Lokale save/load is geversioneerd (formaat 3, migreert formaat 2) en vervangt het bestand atomair; NPC's, wachtende scriptacties en undo/redo-geschiedenis worden niet opgeslagen.
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
