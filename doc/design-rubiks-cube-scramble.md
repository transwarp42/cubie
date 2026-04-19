# Ontwerp: Scramble Architectuur voor Cubie

## 0. Renderingkeuze

De applicatie gebruikt **volledig unlit rendering** (via `StandardMaterial { unlit: true }` op alle materialen). Dit garandeert:

- **Consistente kubuskleuren**: Elke sticker toont exact de gedefinieerde `StickerColor::to_color()` waarde, ongeacht camerahoek.
- **Geen schaduwvariatie**: Geen ambient occlusion, specular highlights of diffuse shading die kleuren beïnvloeden.

**Waarom PBR ongeschikt is**: PBR introduceert lichtafhankelijke kleurvariatie. Een rode sticker zou donkerder lijken in de schaduw en lichter bij directe belichting — onaanvaardbaar voor een puzzel waar kleurherkenning essentieel is.

### Rendering hardening maatregelen

De volgende maatregelen zijn genomen om kleurintegriteit te garanderen:

| Maatregel | Implementatie | Reden |
|-----------|--------------|-------|
| Unlit materialen | `StandardMaterial { unlit: true }` op alle materialen (stickers + cubie body) | Kleuren onafhankelijk van licht |
| Geen lichtbronnen | Geen `DirectionalLight`, geen `AmbientLight` | Elimineert schaduw-artefacten |
| Geen tone mapping | `Tonemapping::None` op camera | Voorkomt kleurverschuiving door post-processing |
| Double-sided stickers | `double_sided: true, cull_mode: None` op sticker materialen | Voorkomt onzichtbare stickers door backface culling na rotatie |
| Sticker elevatie | `STICKER_ELEVATION: 0.005` | Voorkomt z-fighting tussen sticker en cubie body |
| Rotatie snapping | `snap_rotation()` met brute-force 24-oriëntatie matching | Voorkomt floating-point drift na vele rotaties |

---

## 1. Stapsgewijs Ontwikkelplan

### Fase 1: Scramble State & Move Generation
- Random-state scramble via `rcuber` crate (Kociemba two-phase algoritme)
- `ScrambleQueue` resource introduceren
- **Risico**: Solver kan traag zijn bij eerste aanroep (lookup tables initialiseren)
- **Validatie**: Verify dat gegenereerde state geldig is en scramble ≤21 moves bevat

### Fase 2: Scramble Animatie Pipeline
- Sequentiële move-afspeling via bestaande animatie-infrastructuur
- `ActionOrigin::Scramble` variant toevoegen
- **Risico**: Timing/synchronisatie tussen opeenvolgende moves
- **Validatie**: Visuele inspectie; cube eindstate matcht logische state

### Fase 3: UI Integratie
- Scramble knop + bevestigingsdialog
- History reset na voltooiing
- **Risico**: Gebruiker klikt undo tijdens scramble
- **Validatie**: Input blocking tijdens scramble werkt correct

---

## 2. Architectuurschets

### Modulestructuur

```
src/cube/
├── scramble.rs    (nieuw)
├── mod.rs         (+ pub mod scramble)
└── ... (bestaand)
```

### Components

Geen nieuwe components nodig. Bestaande `Cubie` en `Sticker` volstaan.

### Resources

| Resource | Doel |
|----------|------|
| `ScrambleQueue` | Bevat de lijst moves nog af te spelen, plus status (Idle/Active/Confirming) |

### Systems

| System | Verantwoordelijkheid |
|--------|---------------------|
| `spawn_scramble_button` | UI knop spawnen (Startup) |
| `handle_scramble_input` | Klik detectie → confirmation state |
| `handle_scramble_confirmation` | Dialoog interactie → genereer moves, vul queue |
| `process_scramble_queue` | Pakt volgende move uit queue, start animatie |
| `finish_scramble` | Na laatste move: reset history, zet status naar Idle |
| `update_scramble_button` | Visuele enable/disable van de scramble knop |

---

## 3. Scramble Model

### Representatie van cube state

De bestaande `CubeState` resource (27 cubies met grid_position + stickers) is de canonieke representatie. Geen parallelle datastructuur nodig.

### Representatie van moves

Bestaande `CubeMove { axis, layer, clockwise }`. Voor 180° rotaties: twee opeenvolgende identieke moves in de queue.

### Random-state scramble (geïmplementeerd via `rcuber`)

De scramble wordt gegenereerd met het Kociemba two-phase algoritme:

1. `rcuber::generator::Generator::random()` genereert een willekeurige, geldige cube state
   - Valide permutatie van 8 hoekstukken + oriëntatie (mod 3)
   - Valide permutatie van 12 randstukken + oriëntatie (mod 2)
   - Oplosbaarheid gegarandeerd (pariteitscheck, oriëntatiesom)
2. `rcuber::solver::min2phase::Min2PhaseSolver` lost deze state op (random_state → solved)
3. De oplossing wordt **geïnverteerd** (omgekeerde volgorde + inverse van elke move) om de scramble te verkrijgen (solved → random_state)
4. Resultaat: ≤21 moves, WCA-compliant kwaliteit

### Move-mapping (rcuber → CubeMove)

De `rcuber::moves::Move` enum wordt vertaald naar `CubeMove` via `rcuber_move_to_cube_moves()`:

| Standaardnotatie | Axis | Layer | Clockwise | Toelichting |
|------------------|------|-------|-----------|-------------|
| U  | Y | +1 | true  | CW vanuit +Y perspectief |
| U' | Y | +1 | false | CCW vanuit +Y perspectief |
| D  | Y | -1 | false | CW vanuit -Y = CCW vanuit +Y |
| D' | Y | -1 | true  | CCW vanuit -Y = CW vanuit +Y |
| R  | X | +1 | true  | CW vanuit +X perspectief |
| R' | X | +1 | false | CCW vanuit +X perspectief |
| L  | X | -1 | false | CW vanuit -X = CCW vanuit +X |
| L' | X | -1 | true  | CCW vanuit -X = CW vanuit +X |
| F  | Z | +1 | true  | CW vanuit +Z perspectief |
| F' | Z | +1 | false | CCW vanuit +Z perspectief |
| B  | Z | -1 | false | CW vanuit -Z = CCW vanuit +Z |
| B' | Z | -1 | true  | CCW vanuit -Z = CW vanuit +Z |

Voor 180° moves (U2, D2, etc.) worden twee identieke 90° CubeMoves in de queue geplaatst.

### Integratie met ECS en animation pipeline

De `ScrambleQueue` resource bevat een `VecDeque<CubeMove>`. Het `process_scramble_queue` system:
- Checkt of `FaceRotationAnimation.active == false`
- Popt de volgende move uit de queue
- Start animatie met `ActionOrigin::Scramble`
- In `finish_face_rotation`: als origin == Scramble → **niet** naar history schrijven

Na lege queue → `finish_scramble` system reset `ActionHistory` (beide stacks leeg).

---

## 4. Ontwerpbeslissingen

### State Management Strategie: Command-based (bestaand, behouden)

**Keuze**: Command-based (moves als atomaire acties)

- **Waarom ECS-passend**: Moves zijn pure data, systems zijn stateless transformaties. Geen object-state nodig.
- **Performance**: O(1) per move toepassing op `CubeState`. Geen snapshots kopiëren.
- **Geheugen**: Stack van `CubeMove` (12 bytes per move) — verwaarloosbaar.
- **Scramble integratie**: Scramble = reeks moves die gewoon worden afgespeeld maar niet opgeslagen in history. Na afloop: stacks clearen. Simpel en elegant.

### ECS Integratie

- **Actieopslag**: `ScrambleQueue` als Resource (niet Events, want multi-frame lifetime)
- **Stateless systems**: Elk system leest Resources/Components, muteert, klaar. Geen lokale state.
- **Loose coupling**: Systems communiceren alleen via `ScrambleQueue`, `FaceRotationAnimation`, en `ActionHistory`. Geen directe system-naar-system afhankelijkheid.
- **Pipeline opsplitsing**: Elke stap is een apart system. Alle Update systems worden via `.chain()` sequentieel uitgevoerd om deterministische volgorde te garanderen.

### Rotatie-integriteit

Na elke rotatie (inclusief scramble moves) wordt `snap_rotation()` toegepast om de cubie-oriëntatie te snappen naar de dichtstbijzijnde van de 24 geldige kubus-oriëntaties. Dit voorkomt floating-point drift die na vele opeenvolgende rotaties (20+ tijdens scramble) kan leiden tot:

- Visueel scheve cubelets
- Stickers die naar binnen wijzen (onzichtbaar door backface culling)
- Inconsistente grid-posities

De implementatie genereert alle 24 oriëntaties (6 face-richtingen × 4 rotaties per as) en selecteert de dichtstbijzijnde via quaternion dot product.

---

## 5. Dataflow

```
[Scramble Button Click]
    → handle_scramble_input
        leest: Interaction query
        schrijft: ScrambleQueue.status = Confirming

[Confirmation Dialog - "Ja"]
    → handle_scramble_confirmation
        leest: dialog interaction
        schrijft: ScrambleQueue.moves = generate_scramble_moves()
                  (Generator::random() → Min2PhaseSolver::solve() → inverteer oplossing → map naar CubeMove)
        schrijft: ScrambleQueue.status = Active

[Per frame tijdens scramble]
    → process_scramble_queue
        leest: ScrambleQueue, FaceRotationAnimation.active
        schrijft: FaceRotationAnimation (start volgende move)
        schrijft: ScrambleQueue (pop move)

[Animatie per frame]
    → animate_face_rotation (bestaand)
        leest: Time, FaceRotationAnimation
        schrijft: Transform van pivot

[Animatie klaar]
    → finish_face_rotation (bestaand, aangepast)
        leest: FaceRotationAnimation
        schrijft: Cubie grid_positions, CubeState, transforms
        schrijft: NIET naar ActionHistory als origin == Scramble

[Queue leeg na laatste finish]
    → finish_scramble
        leest: ScrambleQueue (leeg + status Active)
        schrijft: ActionHistory (clear both stacks)
        schrijft: ScrambleQueue.status = Idle
```

**Input blocking**: Tijdens `ScrambleQueue.status == Active` worden `handle_undo_redo_input`, `start_face_rotation`, en picking systems geblokkeerd via run-conditions.

---

## 6. Libraries / Tooling

| Optie | Voordelen | Nadelen | Status |
|-------|-----------|---------|--------|
| `rcuber` crate | Random-state scramble, Kociemba two-phase solver, WCA-compliant kwaliteit, ≤21 moves | Grotere dependency, lookup tables initialisatie bij eerste gebruik | **Geïmplementeerd** |
| `rand` crate | Lightweight | Niet meer nodig voor scramble (rcuber bevat eigen randomisatie) | Behouden voor overig gebruik |

---

## 7. Aannames

- De bestaande animatie-pipeline ondersteunt slechts één actieve animatie tegelijk (bevestigd door `animation.active` checks)
- Scramble-snelheid is verhoogd (`duration: 0.1` i.p.v. `0.3`) voor betere UX
- Er is geen parallelle multi-move animatie nodig
- De confirmation dialog is een simpele Bevy UI overlay (geen externe UI library)
- De `rcuber` solver levert altijd een geldige oplossing voor door `Generator::random()` gegenereerde states
- Alle materialen zijn unlit — er zijn geen lichtbronnen in de scene
