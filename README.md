# Cubie — 3D Rubik's Cube in Rust

Een 3D Rubik's Cube viewer gebouwd met [Bevy](https://bevyengine.org/) in Rust.

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange)
![Bevy](https://img.shields.io/badge/Bevy-0.15-232326)

## Functies

- 3×3×3 Rubik's Cube met correcte kleuren per zijde
- Vrij draaibare orbit-camera (linkermuisknop + slepen)
- Zoom met scrollwiel
- Architectuur voorbereid op laagrotaties en animaties

## Vereisten

- [Rust](https://rustup.rs/) (edition 2024)
- macOS, Linux of Windows

## Bouwen en starten

```sh
cargo run
```

Voor een geoptimaliseerde build:

```sh
cargo run --release
```

## Besturing

| Actie | Invoer |
|-------|--------|
| Kubus roteren | Linkermuisknop + slepen |
| Zoomen | Scrollwiel |

## Projectstructuur

```
src/
├── main.rs          # App-configuratie en plugin-registratie
├── camera.rs        # Orbit-camera met muisbesturing
└── cube/
    ├── mod.rs       # Module-declaraties
    ├── model.rs     # Datamodel: CubeState, CubieData, FaceDirection, StickerColor
    └── spawn.rs     # Spawnt cubie-entiteiten met meshes en materialen
```

## Ontwerp

Zie [doc/design-rubiks-cube.md](doc/design-rubiks-cube.md) voor het volledige ontwerpdocument met architectuurbeslissingen, ontwikkelplan en uitbreidbaarheidsstrategie.

## Licentie

MIT
