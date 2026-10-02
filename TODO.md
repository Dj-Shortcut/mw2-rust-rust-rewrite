# TODO — pc-survival met FPS en skaten

Doel: een zelfstandige pc-game in Rust/Bevy, met survival en bouwen geïnspireerd
door Rust, FPS-gunplay en operators geïnspireerd door MW2, en skateboarden
geïnspireerd door Skate 3. Werelden, modellen, materialen, animaties en geluiden
maken we zelf. Originele gamebestanden zijn geen vereiste voor het eindproduct.

**Status op 2 oktober 2026: onafgewerkte ontwikkelbroncode. Geen afgewerkt
speelbaar product en geen releasebinary.** De repository bevat de implementatie
en dient als broncodeoverdracht, met onderstaande verificatiegrenzen.

`[x]` betekent uitsluitend dat de genoemde codecontrole is geslaagd. Het betekent
geen speelbare feature. Een feature is pas klaar voor de speler na integratie,
een uitvoerbare build en verificatie van de volledige gebruikersflow.

## Wat aantoonbaar aanwezig is

| Onderdeel | Code | Controle | Speelbaar geverifieerd |
|---|---|---|---|
| Bouwstukken, kosten, eigendom, upgrades, instorting en opslag | Aanwezig | 8 tijdelijke scenario's geslaagd | Nee |
| Kogel-/mêleeschade aan bouwstukken, voorspelling zonder schade | Aanwezig | 5 simulatieprobes geslaagd | Nee |
| Mapreader SDK-v9 en begrensde parser | Aanwezig | 8 map/corevalidatieprobes en inspectie-CLI geslaagd | Nee, inspectietool |
| Bouwstukken in wereldsnapshots en loopcollisie | Geïntegreerd | Compilercontrole geslaagd | Geen netwerksessie getest |
| Lokale hostbouwbediening en Xbox-controllerbindings | Aanwezig | Compilercontrole geslaagd | Geen controllerflow getest |
| Eigen procedurele hout-/steen-/metaalmaterialen | Aanwezig | WGSL/Naga-validatie geslaagd | Geen GPU-weergave getest |
| MW2/Skate-enginebasis uit publieke rewritecode | Aanwezig | Eerdere launchercontrole geslaagd | Geen complete gameflow getest |
| Zelfstandig starten zonder originele gamebestanden | Startpad aanwezig | Compilercontrole geslaagd; sessiebootstrap geblokkeerd | Nee |
| Eigen terreinmesh en skateobjecteditor | Backend en frontend aanwezig | Bootstrap-compilercontrole geslaagd; uitvoering nog geblokkeerd | Nee |

- [x] Bestaande bouwscenario's, schadeprobes en mapvalidatie uitgevoerd.
- [x] Procedurele materiaalshader geparseerd en gevalideerd.
- [x] Gedownloade bouwtextures en downloadscript uit productpad gehaald.
- [x] Actuele launcher, native frontend en mapreader compileren succesvol.
- [ ] Native executable bouwen en grafische/bedieningsflow zelf verifiëren.

## Actuele blokkade

Het native startpad compileert, maar de eerste uitvoeringsprobe stopt omdat de
gedeelde simulatie een gestart authority-gamescript vereist. Dat moet met onze
eigen sessieregels opgelost worden voordat we bewegen, schieten en de editor als
werkend mogen afvinken. Er is nog geen grafische gameflow geverifieerd.

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
- [ ] Afgewerkt product reviewbaar maken en een geverifieerde speelbare release leveren.

Deze lijst beschrijft de ambitie en ontbrekende integratie. Ze beweert geen
volledige Rust-pariteit of percentage voltooiing; extra subsystemen worden
concreet toegevoegd zodra de gekozen spelregels en implementatie daarom vragen.
