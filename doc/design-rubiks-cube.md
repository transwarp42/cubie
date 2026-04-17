# Ontwerpdocument: 3D Rubik's Cube in Rust

## 1. Gekozen technologie en motivatie

### Bevy Engine (v0.15)

Voor dit project is gekozen voor **Bevy**, een data-driven game engine geschreven in Rust. De motivatie:

- **Entity Component System (ECS)**: Bevy's ECS-architectuur biedt een natuurlijke scheiding tussen data (componenten), logica (systemen) en entiteiten. Dit past perfect bij de eis om rendering, input en kubus-logica te scheiden.
- **Ingebouwde 3D-rendering**: PBR-rendering (Physically Based Rendering) met `wgpu` als backend, zonder handmatig shaders of renderpipelines op te zetten.
- **Pluginsysteem**: Functionaliteit kan modulair worden toegevoegd via plugins, wat uitbreidbaarheid vergemakkelijkt.
- **Geen browser vereist**: Bevy draait native op desktop (Windows, macOS, Linux).
- **Actieve community**: Bevy is het meest actieve Rust game engine-project met uitgebreide documentatie.

**Alternatieven overwogen:**
- `wgpu + winit`: Te laag niveau voor dit project; veel boilerplate nodig voor camerabeheer, input, en meshrendering.
- `macroquad`/`ggez`: Voornamelijk 2D-gericht, minder geschikt voor 3D-kubus.

---

## 2. Architectuur

### Modulestructuur

```
src/
├── main.rs              # App-configuratie, plugin-registratie
├── camera.rs            # Orbit-camera: component, setup en input-systeem
└── cube/
    ├── mod.rs           # Module-declaraties
    ├── model.rs         # Datamodel: CubeState, CubieData, FaceDirection, StickerColor
    └── spawn.rs         # Systeem om cubie-entiteiten te spawnen
```

### Kernstructuren

```
CubeState (Resource)
├── cubies: Vec<CubieData>
│   ├── grid_position: IVec3        // positie in rooster (-1..=1)³
│   └── stickers: Vec<(FaceDirection, StickerColor)>

Cubie (Component)
└── grid_position: IVec3            // koppeling entity ↔ logisch model

OrbitCamera (Component)
├── distance: f32                   // afstand tot oorsprong
├── yaw: f32                        // horizontale draaihoek
├── pitch: f32                      // verticale draaihoek
└── sensitivity: f32                // muisgevoeligheid
```

### Entiteitenhiërarchie (per cubie)

```
CubieEntity [Cubie, Transform, Visibility]
├── Body [Mesh3d(Cuboid), MeshMaterial3d(zwart)]
├── Sticker [Mesh3d(Rectangle), MeshMaterial3d(kleur), Transform]
├── Sticker [...]
└── ...
```

Elke cubie is een parent-entity op de roosterpositie. Kinderen zijn het zwarte lichaam en de gekleurde stickers. Door deze hiërarchie hoeft bij een latere laagrotatie alleen de parent-transform aangepast te worden.

---

## 3. Stapsgewijs ontwikkelplan

### Stap 1: Projectopzet
- Cargo-project met Bevy-dependency
- Basisvenster met achtergrondkleur

### Stap 2: Kubusmodel
- Definieer `FaceDirection`, `StickerColor`, `CubieData`, `CubeState`
- Implementeer `CubeState::solved()` die een 3×3×3 opgeloste kubus genereert
- Wijs kleuren toe op basis van positie: buitenvlakken krijgen standaardkleuren

### Stap 3: Kubus renderen
- Spawn 27 cubie-entiteiten op roosterposities
- Elk cubie-lichaam is een zwarte `Cuboid` (0.9³) met kleine tussenruimte
- Buitenvlakken krijgen gekleurde `Rectangle`-stickers net boven het oppervlak
- Materialen worden vooraf aangemaakt en hergebruikt

### Stap 4: Orbit-camera
- Camera op sferische coördinaten rond de oorsprong
- Muissleep (linkerknop ingedrukt) past yaw/pitch aan
- Scrollwiel regelt zoom (afstand)
- Pitch begrensd om gimbal lock te voorkomen

### Stap 5: Belichting
- Directioneel licht voor diepte-effect
- Ambient light zodat alle vlakken zichtbaar zijn

---

## 4. Uitbreidbaarheid naar zijde-rotaties

De architectuur is bewust opgezet om later eenvoudig laagrotaties toe te voegen:

### Logische laag
- `CubeState` bevat per cubie een `grid_position` en sticker-informatie.
- Bij een laagrotatie (bijv. rechterlaag 90° met de klok mee):
  1. Selecteer cubies waar `grid_position.x == 1`
  2. Roteer hun `grid_position` in het YZ-vlak
  3. Roteer de `FaceDirection` van hun stickers overeenkomstig

### Visuele laag
- Cubie-entiteiten zijn gegroepeerd via de parent-child-hiërarchie.
- Een rotatie-animatie kan worden toegevoegd door:
  1. Een tijdelijke parent-entity aan te maken voor de te roteren laag
  2. De cubie-entiteiten tijdelijk als kinderen hiervan te plaatsen
  3. De parent te animeren (bijv. via `Quat::slerp` over frames)
  4. Na voltooiing de cubies weer los te koppelen en hun logische staat bij te werken

### Toekomstige modules
- `cube/rotation.rs` — Logica voor laagrotaties en face-mappings
- `cube/animation.rs` — Geanimeerde overgangen met interpolatie
- `cube/input.rs` — Muisinteractie voor het selecteren en draaien van lagen

### State management
- `CubeState` als single source of truth voor de logische kubus
- Entiteiten worden gesynchroniseerd met de logische staat na elke rotatie
- Hierdoor is het eenvoudig om undo/redo, scramble, of solver-functionaliteit toe te voegen

---

## 5. Technische details

### Kleurtoewijzing (opgeloste staat)
| Richting   | As   | Kleur   |
|------------|------|---------|
| Up (+Y)    | Y=1  | Wit     |
| Down (-Y)  | Y=-1 | Geel    |
| Front (+Z) | Z=1  | Groen   |
| Back (-Z)  | Z=-1 | Blauw   |
| Right (+X) | X=1  | Rood    |
| Left (-X)  | X=-1 | Oranje  |

### Afmetingen
- **Cubie-lichaam**: 0.9 × 0.9 × 0.9 (tussenruimte van 0.1)
- **Sticker**: 0.82 × 0.82 (iets kleiner dan het vlak, voor bevel-effect)
- **Sticker-offset**: 0.001 boven het cubie-oppervlak (z-fighting voorkomen)

### Camera-standaardinstellingen
- **Afstand**: 8.0
- **Yaw**: π/4 (45°)
- **Pitch**: π/6 (30°)
- **Zoom-bereik**: 4.0 – 15.0
