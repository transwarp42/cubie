# Ontwerp: Scramble Architectuur voor Cubie

## 0. Renderingkeuze

De applicatie gebruikt **unlit rendering** (via `MeshMaterial3d` met onverlichte kleuren). Dit garandeert:

- **Consistente kubuskleuren**: Elke sticker toont exact de gedefinieerde `StickerColor::to_color()` waarde, ongeacht camerahoek of lichtpositie.
- **Geen schaduwvariatie**: Geen ambient occlusion, specular highlights of diffuse shading die kleuren beïnvloeden.

**Waarom PBR ongeschikt is**: PBR introduceert lichtafhankelijke kleurvariatie. Een rode sticker zou donkerder lijken in de schaduw en lichter bij directe belichting — onaanvaardbaar voor een puzzel waar kleurherkenning essentieel is.

---

## 1. Stapsgewijs Ontwikkelplan

### Fase 1: Scramble State & Move Generation
- Random-move scramble implementeren (baseline)
- `ScrambleQueue` resource introduceren
- **Risico**: Move-generatie produceert triviale of redundante sequenties
- **Validatie**: Verify dat gegenereerde moves geen directe herhalingen bevatten

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

### Fase 4: Random-State Scramble (optioneel/toekomstig)
- Kociemba-algoritme integreren via externe crate
- **Risico**: Complexiteit, tabellen laden, performance
- **Validatie**: Gegenereerde state is geldig; oplossing bereikt target state

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

---

## 3. Scramble Model

### Representatie van cube state

De bestaande `CubeState` resource (27 cubies met grid_position + stickers) is de canonieke representatie. Geen parallelle datastructuur nodig.

### Representatie van moves

Bestaande `CubeMove { axis, layer, clockwise }`. Voor 180° rotaties: twee opeenvolgende identieke moves in de queue.

### Random-move scramble (baseline, eerste implementatie)

1. Genereer 20–25 moves
2. Per move: kies random `RotationAxis` + `layer` ∈ {-1, 1} (buitenste lagen) + `clockwise` ∈ {true, false}
3. Constraint: volgende move mag niet dezelfde axis+layer combinatie hebben als vorige
4. 180° wordt gerepresenteerd door dezelfde move tweemaal toe te voegen

Mapping naar standaardnotatie:
- U = Y, layer 1, CW
- D = Y, layer -1, CCW (vanuit +Y perspectief)
- R = X, layer 1, CW
- L = X, layer -1, CCW
- F = Z, layer 1, CW
- B = Z, layer -1, CCW

### Random-state scramble (geavanceerd, toekomstig)

1. Genereer willekeurige permutatie van 8 hoekstukken + oriëntatie (mod 3)
2. Genereer willekeurige permutatie van 12 randstukken + oriëntatie (mod 2)
3. Valideer oplosbaarheid (pariteitscheck, oriëntatiesom)
4. Gebruik Kociemba two-phase solver om move-sequentie te berekenen
5. Resultaat: ≤20 moves (optimaal)

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
- **Pipeline opsplitsing**: Elke stap is een apart system met run-conditions (`scramble_queue.is_active()`, `!animation.active`, etc.)

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

| Optie | Voordelen | Nadelen | Wanneer kiezen |
|-------|-----------|---------|----------------|
| `rand` crate | Simpel, lightweight, genoeg voor random-move | Geen cube-specifieke logica | Altijd (baseline) |
| `kociemba` / eigen two-phase solver | Optimale scrambles (≤20 moves), WCA-compliant | Complexiteit, lookup tables (~20MB), langere opstarttijd | Wanneer competitie-kwaliteit vereist is |
| Geen externe solver, eigen random-move | Geen dependencies, simpel te testen | Niet WCA-compliant, mogelijk bias in distributie | MVP / eerste release |

**Aanbeveling**: Start met random-move scramble via `rand`. Voeg later optioneel een Kociemba-solver toe als feature flag.

---

## 7. Aannames

- De bestaande animatie-pipeline ondersteunt slechts één actieve animatie tegelijk (bevestigd door `animation.active` checks)
- Scramble-snelheid kan verhoogd worden (bijv. `duration: 0.1` i.p.v. `0.3`) voor betere UX
- Er is geen parallelle multi-move animatie nodig
- De confirmation dialog wordt een simpele Bevy UI overlay (geen externe UI library)

