# Procedurele bouwmaterialen

Hout, steen en metaal worden volledig uit eigen shadercode opgebouwd in
`crates/render_gpu/src/drawsurf/buildings.rs`. De runtime gebruikt hiervoor
geen texturebestanden, downloads, afbeeldingsdecoder of texture-atlas.

- Hout: planken, naden, kleurvariatie en langgerekte houtnerf.
- Steen: verspringende blokken, voegen, onregelmatige tinten en poriën.
- Metaal: plaatnaden, klinknagels, geborstelde variatie en lichte corrosie.

Geometrie komt uit de bouwstukgrenzen van `rust_building`; UV's bepalen de
detailschaal en wereldposities variëren de materialen per oppervlak.
Fragmentderivaten verminderen fijne details op afstand. Het renderpad gebruikt
de bestaande scene-depth en een eenvoudige diffuse lichtberekening.

Deze materialen zijn eigen inhoud van de rewrite. Ze zijn nog niet visueel
afgestemd op een oorspronkelijke game; shadersyntax en runtime-integratie
worden afzonderlijk gecontroleerd. Er worden geen originele game-assets
meegeleverd.
