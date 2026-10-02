# PC survival / FPS / Skate

We bouwen een zelfstandige **pc-survivalgame in Rust en Bevy**, met survival
en bases geïnspireerd door Rust, gunplay en operators geïnspireerd door MW2,
en skateboarden geïnspireerd door Skate 3. De wereld en gamecontent maken we
zelf: modellen, materialen, animaties en geluiden.

Het eindproduct moet zonder originele gamebestanden starten. Het gebruikt
publieke engine-/rewritecode en eigen content. De volledige featurelijst en
het onderscheid tussen aanwezige code, controles en speelbaarheid staan in
[TODO.md](TODO.md).

**Status: onafgewerkte ontwikkelbroncode. Er is nog geen afgewerkt speelbaar
product of releasebinary.** De repository dient als broncodeoverdracht en
voortgangsoverzicht. Wij verifiëren de implementatie zelf voordat we een
speelbare release leveren.

## Huidige implementatie

- Rust/Bevy-enginebasis met IW4L en de publieke MW2/Skate-integratie uit
  [2010-rust-rewrite-mashup](https://github.com/chasmlol/2010-rust-rewrite-mashup).
- Bouwkern: funderingen, vloeren, muren, deuren, eigendom, materialen,
  kosten/upgrades, ondersteuning/instorting en gevalideerde opslag.
- Autoritatieve kogel-/mêleeschade, loopcollisie en bouwstaat in wereldsnapshots.
- Lokale hostbouwbediening voor toetsenbord/muis en Xbox-controller op pc.
- Eigen procedurele hout-, steen- en metaalmaterialen zonder texturedownloads.
- Optionele SDK-v9-mapinspectietool; die is geen speelbare wereldimport.

Het zelfstandige startpad, een klein eigen eilandterrein en een skateobjecteditor
zijn aanwezig in de code. De editor bevat ramps, quarterpipes, rails, trappen,
platforms en funboxes, met plaatsing, rotatie, verwijderen, undo/redo en opslag.
De native frontend compileert; de eerste uitvoeringscontrole stopt omdat de
gedeelde simulatie een geladen en gestart GSC-programma vereist. De native
gameflow en GPU-weergave zijn nog niet geverifieerd. Gathering, crafting,
inventory/survival, een volledige wereld, skate-integratie en multiplayerloops
moeten nog worden afgemaakt. [TODO.md](TODO.md) houdt deze onderdelen apart van
reeds uitgevoerde codecontroles.

## Bouwen en starten

Installeer Rust stable en de [systeemdependencies](docs/BUILD.md). Voor
compilercontrole vanuit de repository:

```bash
cargo check -p launcher -p rust_maps
```

De launcher kiest standaard het zelfstandige native startpad; `game` kiest
hetzelfde pad. Dit pad is nog geblokkeerd door de sessiebootstrap hierboven.
De definitieve startinstructies volgen na uitvoeringsverificatie. Er is nog
geen kant-en-klare executable.
De bestaande [RUN.md](docs/RUN.md) en [SKATE.md](docs/SKATE.md) beschrijven de
upstreamgame-importmodi; hun releases en assetvereisten zijn niet de levering
of vereisten van de zelfstandige game die we hier bouwen.

## Verificatie en herkomst

Tijdelijke probes controleren bouwen, kosten/eigendom, deurcollisie,
instorting/opslag, schade en mapvalidatie. De inspectie-CLI is uitgevoerd op
een synthetische SDK-map. De nieuwe procedurele WGSL-shader is geparseerd en
gevalideerd. `cargo check -p launcher -p rust_maps` is geslaagd, inclusief de
native frontend. Dit is codebewijs; een uitvoerbare releasebuild en
gameplay-/GPU-/multiplayerverificatie zijn nog nodig.

Upstreamdocumentatie staat in [UPSTREAM.md](docs/UPSTREAM.md); credits en
licenties staan in [NOTICE](NOTICE) en [LICENSE](LICENSE). Oorspronkelijke
gamecontent wordt niet meegeleverd.
