# Bouwsysteem en bediening

Huidige implementatie: lokale listen host, op geladen MW2-map met levende
speler. Clients ontvangen bouwstukken via wereldsnapshots maar kunnen nog
geen bouwverzoeken sturen. Beide peers moeten dezelfde protocolversie hebben.

Open de console met `~`, spawn indien nodig met `spawn 0`, wacht tot de
spelmodus beweging vrijgeeft en geef ontwikkelresources:

```text
build grant 10000 10000 10000
build anchor
```

Resources verzamelen is nog niet geïmplementeerd. `anchor` plaatst het raster
op geschikte grond voor de speler; een raster met bouwstukken kan niet verhuizen.

| Actie | Toetsenbord/muis | Xbox-controller op pc |
|---|---|---|
| Bouwmodus aan/uit | B | Back vasthouden + D-pad omhoog |
| Bouwstuk kiezen | Q / E | D-pad links / rechts |
| Muur/deur draaien | R | LT |
| Plaatsen op gericht oppervlak | Linkermuisknop | RT |
| Deur openen/sluiten | Rechtermuisknop op frame | X op frame |
| Opslaan als autosave | F | Y |

Selectie: fundering, vloer, muur, deurkozijn inclusief deur. Bouwmodus schakelt
schieten/ADS op de gereserveerde knoppen uit. Console, focusverlies en
scriptmenu's blokkeren bouwacties. Tijdens skaten is bouwen uitgeschakeld.
De bediening en het nieuwe renderpad zijn nog niet in live gameplay geverifieerd.

Console: `build status`; `build place foundation 0 0 0 0`;
`build place wall 0 0 0 0`; `build upgrade 1 stone`; `build door 2`;
`build demolish 1`; `build save mijnbasis`; `build load mijnbasis`.
Plaatsargumenten zijn type, raster-x, raster-y, verdieping en muur-as (0/1).
Funderingen kosten 200 hout; vloeren/muren 100; deurkozijnen 150.
Upgrades kosten tweemaal dit aantal steen of eenmaal dit aantal metaal.

Opslag: `iw4l-artifacts/buildings/<map-content-digest>/<naam>.json`.
Naam: maximaal 32 letters/cijfers/underscores/streepjes. Laden valideert schema,
stuk-ID's, gezondheid, bezette sockets, ondersteuning en bestandslimiet.
Eigendom beperkt deurbediening, upgrades en slopen. Een stuk zonder steun
stort in na verwijdering van dragende stukken; dit is een vereenvoudigd model.
Autoritatieve kogels en mêlee beschadigen bouwstukken; voorspelling doet dat niet.

Nog af te werken: plaatsingspreview, gathering, crafting, volledige Rust-
stabiliteit, clientbouwverzoeken, gamepadbediening voor upgrades/slopen,
Skate-collisie en verificatie met echte gamebestanden.
