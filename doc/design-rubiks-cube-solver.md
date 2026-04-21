# Ontwerpdocument: Solver Architectuur voor Cubie

## 0. Renderingkeuze

### Gekozen aanpak: unlit rendering via StandardMaterial

Het project gebruikt `StandardMaterial` met `unlit: true` voor alle materialen. Deze keuze blijft ongewijzigd voor de solver-uitbreiding en is essentieel voor de correcte werking ervan.

### Waarom unlit cruciaal is voor de solver

De solver moet de visuele kubusstaat uitlezen door de sticker-kleuren en -oriëntaties te detecteren. Met `unlit: true` is de weergegeven kleur uitsluitend bepaald door `base_color`, onafhankelijk van lichtcondities. Dit garandeert:

- **Deterministische kleurdetectie**: Dezelfde fysieke kubusstaat levert altijd dezelfde kleur-mapping, ongeacht camerahoek of lichtpositie
- **Geen licht-artefacten**: Geen diffuse shading of specular highlights die de kleurherkenning verstoren
- **Betrouwbare facelet string generatie**: Het rcuber solver algoritme verwacht exacte kleur-assignments; elke afwijking resulteert in een ongeldige cube state

### Relevantie voor solver

De solver leest de visuele kubusstaat uit door:
1. Voor elke cubie de `Transform.rotation` te lezen (cumulatieve rotatie sinds start)
2. De initiële sticker-normalen te roteren met deze quaternion
3. De resulterende richting te snappen naar een FaceDirection
4. De sticker-kleur te mappen naar een rcuber facelet character

Dit proces is alleen deterministisch als materiaal-kleuren stabiel zijn. PBR zou kleur-variatie introduceren die niet-reproduceerbare facelet strings oplevert, waardoor de solver faalt.

---

## 1. Stapsgewijs Ontwikkelplan

### Fase 1: Visuele state extractie

**Doel**: De kubusstaat uitlezen vanaf de visuele entity transforms en converteren naar een rcuber-compatibele facelet string.

**Werkzaamheden**:
- Implementeer `cubies_to_face_string()`: voor elke cubie, roteer de initiële sticker-normalen met `Transform.rotation`, bepaal de huidige facing direction, map kleuren naar rcuber characters (U/R/F/D/L/B)
- Bouw de 54-character facelet string in de correcte rcuber-volgorde: U-face (9), R-face (9), F-face (9), D-face (9), L-face (9), B-face (9)
- Implementeer `detect_center_mapping()` om middelste laag rotaties af te handelen (zie fase 5)

**Risico's**:
- Floating-point drift na vele rotaties kan leiden tot incorrecte FaceDirection snapping. Mitigatie: de bestaande `snap_rotation()` functie voorkomt dit reeds.
- Facelet string volgorde moet exact de rcuber conventie volgen; één verkeerde index = onoplosbare state.

**Validatie**: 
- Opgeloste kubus moet facelet string "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB" opleveren
- Na één R-move: verifieer dat de correcte 20 facelets veranderd zijn

### Fase 2: Solver integratie via rcuber

**Doel**: De facelet string omzetten naar een CubieCube en oplossen met Kociemba two-phase algoritme.

**Werkzaamheden**:
- Convert facelet string naar `rcuber::facelet::FaceCube`
- Convert `FaceCube` naar `rcuber::cubie::CubieCube` (validatie van pariteit, oriëntatie)
- Roep `Min2PhaseSolver::solve()` aan voor optimale oplossing (≤20 moves God's Number, typisch ≤22 HTM)
- Map rcuber moves terug naar `CubeMove` via de bestaande `rcuber_move_to_cube_moves()` functie

**Risico's**:
- Solver kan falen als kubusstaat ongeldig is (verkeerde pariteit, onmogelijke oriëntaties). Dit gebeurt bij handmatig bewerken van stickers of na bugs in rotatie-logica.
- Eerste solve na opstarten kan traag zijn (pruning tables laden). Mitigatie: UI feedback + lazy initialization.

**Validatie**:
- Solve een kubus na scramble; verifieer visueel dat het opgelost is
- Solve een kubus met één verkeerde edge flip → moet falen met duidelijke error

### Fase 3: Solve queue en animatie pipeline

**Doel**: De solver-moves sequentieel afspelen via dezelfde animatie-infrastructuur als scramble, voorafgegaan door een visuele scan-animatie met return-to-start.

**Werkzaamheden**:
- Introduceer `SolveQueue` resource: status (Idle/Scanning/Active) + `VecDeque<CubeMove>`
- Introduceer `ScanAnimation` resource: tracking voor visuele "fotografeer" effect + beginpositie
- Implementeer scan-animatie met 3 fases:
  1. **Scan faces**: Beweeg camera orthogonaal voor elke zijde (6 × 0.5s = 3.0s)
  2. **Camera flash**: Wit flash-effect bij elke zijde (6 × 150ms)
  3. **Return to start**: Beweeg camera terug naar beginpositie (0.5s)
- Implementeer `process_solve_queue`: popt moves en start animaties met `ActionOrigin::Solve`
- Modificeer `finish_face_rotation`: bij `ActionOrigin::Solve` → wél naar history schrijven (anders dan scramble)

**Risico's**:
- Scan + return + solve duurt lang (3.0s scan + 0.5s return + 20 moves @ 0.3s = 9.5+ seconden). Mitigatie: visueel interessant, gebruiker ziet progressie.
- Return-to-start slerp kan visueel onnatuurlijk zijn bij grote rotaties. Mitigatie: ease-out cubic voor smooth interpolatie.
- Gebruiker klikt undo tijdens scan → blokkering via status check.

**Validatie**:
- Verifieer dat scan-animatie alle 6 zijden toont met flash
- Verifieer dat kubus exact terugkeert naar beginpositie (visuele check)
- Verifieer dat solve-moves correct uitvoeren na return
- Verifieer dat solve-moves in history verschijnen (undo moet werken)

### Fase 4: UI knop en state management

**Doel**: Solve-knop toevoegen met enable/disable logica.

**Werkzaamheden**:
- Spawn Solve-knop links van Scramble-knop
- Implementeer `handle_solve_input`: bij klik → run solver, vul queue
- Implementeer `update_solve_button`: dim knop tijdens solve/scramble/animation
- Implementeer `finish_solve`: na lege queue → status = Idle

**Risico's**:
- Knop positionering: huidige layout is vol. Solve-knop moet tussen Redo en Scramble.
- Solve tijdens reeds opgeloste kubus → lege move queue, maar geen error. Acceptabel gedrag.

**Validatie**:
- Verifieer dat knop disabled is tijdens solve en scramble
- Verifieer dat solve niks doet op opgeloste kubus

### Fase 5: Middelste laag rotatie ondersteuning

**Doel**: Solver moet werken na rotatie van middelste lagen (layer 0), die centrum-pieces verplaatsen.

**Werkzaamheden**:
- Implementeer `detect_center_mapping()`: detecteer welke kleuren op de 6 centrum-posities zitten
- Pas `sticker_color_to_rcuber_char()` aan om dynamische kleur-mapping te gebruiken
- Voorbeeld: als Red centrum op Up-positie zit, map Red → 'U' (in plaats van White → 'U')

**Risico's**:
- rcuber verwacht vaste centrum-kleuren. Middelste laag rotaties breken deze assumptie. Zonder mapping: "could not build CubieCube" error.
- Complexe mapping-logica kan bugs introduceren bij meerdere middelste laag rotaties.

**Validatie**:
- Scramble → draai middelste horizontale laag → solve → moet werken zonder errors
- Draai middelste Y, X, en Z laaigen in combinatie → solve → moet correct centrum-mapping detecteren

---

## 2. Architectuurschets

### Modulestructuur

```
src/cube/
├── solver.rs      (nieuw)
├── mod.rs         (+ pub mod solver)
└── ... (bestaand)
```

### Components

UI marker component: `SolveButton`. Bestaande `Cubie` en `Sticker` volstaan voor kubus-state extractie.

### Resources

| Resource | Doel |
|----------|------|
| `SolveQueue` | Bevat de lijst solve-moves nog af te spelen, plus status (Idle/Scanning/Active) |
| `ScanAnimation` | Tracking voor visuele scan-animatie: welke zijde, elapsed time, flash state, opgeslagen orbit state (yaw/pitch/distance), from_yaw/from_pitch voor smooth interpolatie, return-to-start state |

### Systems

| System | Verantwoordelijkheid |
|--------|---------------------|
| `spawn_solve_button` | Solve-knop spawnen + FlashOverlay UI element (Startup) |
| `handle_solve_input` | Knop klik detectie → zet status naar Scanning |
| `start_scan_animation` | Start scan-animatie: sla camera orbit state (yaw, pitch, distance) op als beginpositie |
| `animate_scan` | Animeer scan: beweeg camera orthogonaal voor elke zijde (6 faces × 0.5s), trigger flash bij 90%; na face 6: beweeg camera terug naar opgeslagen orbit positie (0.5s) |
| `animate_camera_flash` | Animeer wit flash-overlay: fade in (50ms) + fade out (100ms) bij elke zijde |
| `finish_scan_animation` | Wacht tot camera return compleet → herstel orbit state exact (snap), read visual state, run solver, vul queue, zet status naar Active |
| `process_solve_queue` | Pakt volgende move uit queue, start animatie met `ActionOrigin::Solve` |
| `finish_solve` | Na lege queue: zet status naar Idle |
| `update_solve_button` | Visuele enable/disable van knop op basis van states |

### Helper functions

| Function | Doel |
|----------|------|
| `compute_solve_moves()` | Roept `cubies_to_face_string()`, valideert, roept rcuber solver aan, returned `VecDeque<CubeMove>` |
| `cubies_to_face_string()` | Leest entity transforms, bouwt 54-char facelet string met centrum-mapping |
| `detect_center_mapping()` | Detecteert welke kleuren op centrum-posities zitten, returned `HashMap<StickerColor, char>` |
| `sticker_color_to_rcuber_char_mapped()` | Converteert kleur → facelet char via centrum-mapping |

---

## 3. Solver Model

### Representatie van cube state

De visuele kubusstaat (entity `Transform.rotation` + `Sticker` components) is de bron van waarheid voor de solver. De solver leest deze uit en converteert naar rcuber formaat. Er is géén aparte "logische state" voor de solver; `CubeState` wordt gebruikt voor rotatie-logica maar niet voor solver-input.

**Waarom visuele state?** De gebruiker kan handmatig draaien zonder dat `CubeState` wordt bijgewerkt (bijv. bugs, direct entity manipulation in debug mode). De solver moet altijd de actuele visuele toestand oplossen, niet de veronderstelde logische toestand.

### Facelet string formaat (rcuber conventie)

Een 54-character string, in volgorde: URFDLB faces (9 facelets per face).
- Positie 0-8: U-face (Up), van back-left naar front-right (rij voor rij, top naar bottom)
- Positie 9-17: R-face (Right), van top-back naar bottom-front
- Positie 18-26: F-face (Front), van top-left naar bottom-right
- Positie 27-35: D-face (Down), van front-left naar back-right
- Positie 36-44: L-face (Left), van top-back naar bottom-front
- Positie 45-53: B-face (Back), van top-right naar bottom-left (gemirrord)

Elk facelet is een character: U (white), R (red), F (green), D (yellow), L (orange), B (blue).

**Centrum-conventie (vast in rcuber):**
- U-centrum = White (Up face)
- R-centrum = Red (Right face)
- F-centrum = Green (Front face)
- D-centrum = Yellow (Down face)
- L-centrum = Orange (Left face)
- B-centrum = Blue (Back face)

### Middelste laag rotatie problematiek

Een standaard 3x3 Rubik's Cube heeft vaste centra: ze kunnen niet ten opzichte van elkaar bewegen. De rcuber solver gaat hiervan uit. Wanneer de middelste laag (layer 0) wordt gedraaid:

1. De 6 centrum-cubies (op posities zoals `IVec3::new(0,1,0)`, `IVec3::new(1,0,0)`, etc.) verplaatsen
2. Voorbeeld: na middelste Y-laag 90° CW: Red centrum gaat van Right naar Back positie
3. De facelet string zou nu "Red sticker op Up face" kunnen bevatten → ongeldige state voor rcuber

**Oplossing: dynamische centrum-mapping**
- Detecteer welke kleur zich nu op elke centrum-positie bevindt
- Map deze kleur naar het corresponderende facelet character voor die positie
- Voorbeeld: als Red op Up-positie → Red stickers krijgen 'U' character i.p.v. 'R'

Dit "normaliseert" de kubus naar een rcuber-geldige staat waarbij de centra virtueel gefixeerd zijn op hun verwachte posities.

### Kociemba two-phase algoritme (via rcuber)

De `rcuber` crate implementeert Herbert Kociemba's two-phase algoritme:

**Phase 1**: Reduce cube to G1 subgroup (good edges) — max 12 moves
- Alle edge oriëntaties correct (geen flipped edges)
- Alle hoek oriëntaties correct
- 4 specifieke edges in hun slice

**Phase 2**: Solve G1 to identity — max 18 moves
- Permutatie van hoeken
- Permutatie van edges
- Totaal: max 30 moves (in praktijk meestal ≤22)

**Pruning tables**: rcuber gebruikt pre-computed lookup tables voor snelle search. Deze worden lazy-initialized bij eerste `solve()` aanroep.

### Move-mapping (rcuber → CubeMove)

Hergebruikt de bestaande `rcuber_move_to_cube_moves()` uit `scramble.rs`. Geen nieuwe mapping nodig.

### Integratie met ECS en animation pipeline

De `SolveQueue` resource werkt identiek aan `ScrambleQueue`:
- `VecDeque<CubeMove>` bevat de oplossing
- `process_solve_queue` system popt moves en start animaties
- Verschil met scramble: solve-moves worden WÉL naar `ActionHistory` geschreven (via `ActionOrigin::Solve`)

**Waarom solve-moves in history?** De gebruiker moet solve-moves kunnen undo-en. Solve is niet een "reset naar clean state" zoals scramble; het is een reeks acties die de gebruiker mogelijk wil terugdraaien om een tussenstap te inspecteren.

---

## 4. Ontwerpbeslissingen

### State Management Strategie: Visual state extraction

**Keuze**: Lees de kubusstaat uit vanaf entity transforms, niet vanuit `CubeState`

**Waarom**:
- Robustness: visual state is de "ground truth" die de gebruiker ziet
- Tolerantie voor bugs: als logische state en visuele state desynchroniseren (bijv. door een bug in rotatie-logica), lost de solver alsnog de visuele kubus op
- Testbaarheid: solver kan getest worden door handmatig entities te manipuleren zonder `CubeState` te gebruiken

**Trade-off**: Iets langzamer dan direct `CubeState` lezen (moet alle 27 cubies querien + transform rotaties toepassen). In praktijk verwaarloosbaar (< 1ms).

### ECS Integratie

- **Queue als Resource**: `SolveQueue` (niet Events, want multi-frame lifetime)
- **Stateless systems**: Elk system leest Resources/Components, muteert, klaar. Geen lokale state.
- **Loose coupling**: Solve-systemen communiceren alleen via `SolveQueue`, `FaceRotationAnimation`, en `ActionHistory`
- **Pipeline hergebruik**: Solve gebruikt exact dezelfde animatie-pipeline als reguliere moves en scramble

### Solver error handling

Drie failure modes:

1. **Invalid facelet string**: Facelet string voldoet niet aan rcuber formaat-eisen (bijv. niet 9 van elke kleur). Oorzaak: bug in `cubies_to_face_string()`.
   - **Handling**: Log warning, return lege move queue, knop blijft enabled
   
2. **Could not build CubieCube**: Facelet string is valide qua formaat maar representeert onoplosbare state (verkeerde pariteit, onmogelijke oriëntaties). Oorzaak: kubus is handmatig verkeerd geassembleerd of middelste laag rotaties zonder centrum-mapping.
   - **Handling**: Log warning met facelet string, return lege move queue
   
3. **Cube already solved**: `CubieCube::default()` check. Niet echt een error, maar vermijdt onnodige solve-animatie.
   - **Handling**: Return lege move queue, geen log (normaal gedrag)

Alle errors zijn "graceful": geen panics, geen crashes. Knop blijft functioneel.

### Animatie-snelheid voor solve

- **Reguliere moves**: 0.3s (smooth, gebruiker kan volgen)
- **Scramble moves**: 0.1s (snel, veel moves)
- **Solve moves**: 0.3s (standaard — gebruiker wil oplossing zien)
- **Undo/redo**: 0.2s (sneller dan regulier, langzamer dan scramble)

**Keuze voor solve**: Behoud 0.3s (zoals reguliere moves). De gebruiker wil de solve-stappen kunnen volgen voor educatieve waarde. Een snelle solve (0.1s) zou te snel zijn om te begrijpen.

---

## 5. Dataflow

```
[Solve Button Click]
    → handle_solve_input
        leest: Interaction query, SolveQueue.status, ScrambleQueue.status, FaceRotationAnimation.active
        schrijft: SolveQueue.status = Scanning

[Scan Animation Start]
    → start_scan_animation
        leest: SolveQueue.status == Scanning, ScanAnimation.active, OrbitCamera
        schrijft: ScanAnimation.active = true
                  ScanAnimation.saved_orbit_state = (yaw, pitch, distance)
                  ScanAnimation.from_yaw = orbit.yaw (beginpositie)
                  ScanAnimation.from_pitch = orbit.pitch

[Per frame tijdens scan — 6 zijden × 0.5s]
    → animate_scan
        leest: Time, ScanAnimation (current_face, elapsed, from_yaw, from_pitch)
        schrijft: OrbitCamera.yaw/pitch (ease-in-out cubic naar FACE_YAW[i]/FACE_PITCH[i])
                  Camera Transform (positie berekend uit orbit, look_at Vec3::ZERO)
                  ScanAnimation.elapsed += delta
                  Bij 90%: flash_triggered = true, flash_active = true
                  Na voltooiing: from_yaw/from_pitch = target, current_face++
    
    → animate_camera_flash (parallel)
        leest: ScanAnimation.flash_active, flash_elapsed
        schrijft: FlashOverlay BackgroundColor (alpha fade in/out)
                  FlashOverlay Visibility (show/hide)
                  Bij 150ms: flash_active = false

[Na face 6 — return to start × 0.5s]
    → animate_scan (continue)
        leest: current_face >= 6
        schrijft: returning_to_start = true, elapsed = 0
                  from_yaw = FACE_YAW[5], from_pitch = FACE_PITCH[5]
        leest: Time, saved_orbit_state
        schrijft: OrbitCamera.yaw/pitch (ease-in-out cubic terug naar saved state)
                  Camera Transform (positie + look_at)

[Return voltooid — camera terug op beginpositie]
    → finish_scan_animation
        leest: ScanAnimation.returning_to_start == true, elapsed >= duration
        schrijft: OrbitCamera exact op saved_orbit_state (snap)
                  Camera Transform (exact op opgeslagen positie)
        roept: cubies_to_face_string()
            └→ detect_center_mapping()  // voor layer 0 rotaties
            └→ sticker_color_to_rcuber_char_mapped()
        roept: FaceCube::try_from(facelet_string)  // validatie
        roept: CubieCube::try_from(&face_cube)     // pariteit check
        roept: Min2PhaseSolver::solve()            // Kociemba algoritme
        roept: rcuber_move_to_cube_moves()         // mapping
        schrijft: SolveQueue.moves = solution
                  SolveQueue.status = Active (of Idle als geen moves)
                  ScanAnimation = reset

[Per frame tijdens solve]
    → process_solve_queue
        leest: SolveQueue, FaceRotationAnimation.active
        schrijft: FaceRotationAnimation (start volgende move met ActionOrigin::Solve)
        schrijft: SolveQueue (pop move)

[Animatie per frame]
    → animate_face_rotation (bestaand)
        leest: Time, FaceRotationAnimation
        schrijft: Transform van pivot

[Animatie klaar]
    → finish_face_rotation (bestaand, hergebruikt)
        leest: FaceRotationAnimation
        schrijft: Cubie grid_positions, CubeState, transforms
        schrijft: ActionHistory (omdat origin == Solve)

[Queue leeg na laatste finish]
    → finish_solve
        leest: SolveQueue (leeg + status Active)
        schrijft: SolveQueue.status = Idle
```

**Input blocking**: Tijdens `SolveQueue.status == Active`:
- Scramble knop disabled
- Reset knop disabled
- Solve knop disabled
- Reguliere picking en undo/redo werken NIET (geblokkeerd via animation.active check)

### Centrum-mapping dataflow (middelste laag rotaties)

```
[Middelste laag gedraaid]
    → Centra (bijv. IVec3::new(0,1,0)) bevatten nu andere kleuren

[Solve button click]
    → cubies_to_face_string()
        └→ detect_center_mapping(lookup)
            voor elk centrum-positie (0,±1,0), (±1,0,0), (0,0,±1):
                vind sticker op die positie met FaceDirection = Up/Down/etc.
                map kleur → facelet char voor die positie
            return HashMap<StickerColor, char>
        
        voor elke facelet in scan order:
            lookup sticker kleur op grid_position + face_direction
            convert kleur → char via center_map
            append to facelet string
```

Voorbeeld:
- Middelste Y-laag 90° CW gedraaid
- Red centrum nu op Up-positie (0,1,0)
- `detect_center_mapping()` returned: { Red: 'U', Green: 'R', Orange: 'D', Blue: 'L', White: 'F', Yellow: 'B' }
- Alle Red stickers worden 'U', alle Green stickers 'R', etc.
- rcuber ziet een geldige kubus met "Red als Up centrum"

---

## 6. Libraries / Tooling

| Library | Gebruik | Voordelen | Nadelen |
|---------|---------|-----------|---------|
| `rcuber` | Kociemba two-phase solver, CubieCube validatie | Bewezen algoritme, WCA-compliant, ≤22 moves | Grote dependency (~500KB), pruning tables initialization |
| `bevy` v0.15 | ECS, UI, animatie | Volledige infrastructuur aanwezig | N/A |

### rcuber crate details

- **Version**: 0.4.0 (huidige versie, stabiel)
- **Modules gebruikt**:
  - `rcuber::cubie::CubieCube`: Interne kubus-representatie
  - `rcuber::facelet::FaceCube`: Facelet string → CubieCube conversie
  - `rcuber::solver::min2phase::Min2PhaseSolver`: Two-phase algoritme
  - `rcuber::moves::Move`: Standaard notatie (U, R, F, D, L, B + ', 2)
- **Pruning tables**: ~1.5MB, lazy loaded bij eerste solve (200-400ms)

---

## 7. Visuele State Extractie Details

### Transform.rotation naar FaceDirection mapping

Voor elke sticker:
1. Lees initiële `face_direction` (opgeslagen in `Sticker` component)
2. Lees ouder-cubie's `Transform.rotation` (cumulatieve rotatie)
3. Bereken `rotated_normal = rotation * face_direction.normal()`
4. Snap naar dichtstbijzijnde as via `FaceDirection::from_normal()`

**from_normal implementatie**:
```rust
if abs(x) > abs(y) && abs(x) > abs(z) {
    if x > 0.0 { Right } else { Left }
} else if abs(y) > abs(z) {
    if y > 0.0 { Up } else { Down }
} else {
    if z > 0.0 { Front } else { Back }
}
```

Deze max-component snapping is robuust tegen floating-point drift (tot ~0.3 drift = 17° off-axis).

### Face scan order (rcuber conventie)

```rust
// U face: back-left naar front-right, rij voor rij
(-1,1,-1), (0,1,-1), (1,1,-1),  // top row (z=-1)
(-1,1, 0), (0,1, 0), (1,1, 0),  // middle row
(-1,1, 1), (0,1, 1), (1,1, 1),  // front row (z=1)

// R face: top-back naar bottom-front
(1,1, 1), (1,1, 0), (1,1,-1),   // top row
(1,0, 1), (1,0, 0), (1,0,-1),   // middle row
(1,-1,1), (1,-1,0), (1,-1,-1),  // bottom row

// ... (F, D, L, B volgen)
```

**Kritiek detail**: B-face is horizontaal gemirrord (right-to-left scan) omdat we vanaf de achterkant kijken. Dit is de rcuber conventie en MOET exact gevolgd worden.

### Centrum-posities (voor detect_center_mapping)

```rust
Up centrum:    IVec3::new( 0,  1,  0)  → FaceDirection::Up
Down centrum:  IVec3::new( 0, -1,  0)  → FaceDirection::Down
Right centrum: IVec3::new( 1,  0,  0)  → FaceDirection::Right
Left centrum:  IVec3::new(-1,  0,  0)  → FaceDirection::Left
Front centrum: IVec3::new( 0,  0,  1)  → FaceDirection::Front
Back centrum:  IVec3::new( 0,  0, -1)  → FaceDirection::Back
```

Voor elke positie: vind de sticker wiens `current_direction` matcht de verwachte FaceDirection, lees de kleur, map kleur → character voor die face.

### Scan Animatie Details

**3-Fase Process:**

1. **Face Scanning (3.0s totaal)**
   - 6 faces × 0.5s per face
   - **Camera-only animatie**: de kubus roteert niet; alleen de camera beweegt
   - Bij 90% completion: trigger camera flash (150ms wit overlay)
   - **Camera yaw/pitch targets per face** (afstand blijft constant, geen zoom):
     - Front (Z+): yaw=0, pitch=0
     - Right (X+): yaw=+90°, pitch=0
     - Back (Z-): yaw=180°, pitch=0
     - Left (X-): yaw=-90°, pitch=0
     - Up (Y+): yaw=0, pitch=+89.4° (bijna verticaal)
     - Down (Y-): yaw=0, pitch=-89.4°
   - Camera kijkt altijd naar centrum (Vec3::ZERO)
   - Smooth ease-in-out cubic interpolatie tussen posities
   - Orbit camera system is geblokkeerd tijdens scanning

2. **Return to Start (0.5s)**
   - Na face 6 (Down): camera interrpoleert terug naar opgeslagen orbit positie
   - Orbit state (yaw, pitch, distance) wordt exact hersteld (snap op einde)
   - Smooth ease-in-out cubic interpolatie
    
3. **Solver Execution**
   - Start alleen NA complete return
   - Camera staat weer in orbit positie zoals voor scan
   - Orbit camera system wordt weer geactiveerd
   - Visual state extraction leest correcte transforms

**Waarom alleen camera bewegen (geen kubus rotatie)?**
- **Zuivere 2D weergave**: Bij orthogonale camerastand is slechts één zijde zichtbaar; andere zijden zijn op 90° en vallen buiten beeld
- **Geen reparenting overhead**: Geen pivot entity, geen ouder-kind aanpassing van 27 cubies
- **Constante afstand**: Alleen yaw/pitch veranderen, distance blijft intact (geen zoom)
- **Simpeler state**: Geen `initial_rotation`, `pivot_entity` of cubie reparenting nodig

**Waarom return-to-start?**
- Voorkomt verwarring: camera terug naar beginpositie
- Consistentie: solve-animatie start met bekende setup
- Visuele continuïteit: gebruiker ziet duidelijk einde van scan-fase


---

## 8. Aannames

1. Alle materialen blijven `unlit: true` (essentieel voor kleur-stabiliteit)
2. `snap_rotation()` voorkomt floating-point drift bij transform oriëntaties
3. Solve-animatie blokkeert alle andere input (consistent met scramble)
4. rcuber solver faalt nooit op geldige states (aanname van library)
5. Middelste laag rotaties komen voor; centrum-mapping is niet optioneel
6. Gebruiker wil solve-moves kunnen undo-en (daarom ActionOrigin::Solve → history)

## 9. Trade-offs

| Keuze | Voordeel | Nadeel |
|-------|----------|--------|
| Visual state i.p.v. CubeState | Robuust tegen bugs, test visual truth | Langzamer (moet alle entities querien) |
| Solve-moves in history | Gebruiker kan undo, inspect intermediate | Lange undo-stack na meerdere solves |
| 0.3s animatie voor solve | Educatief, gebruiker kan stappen volgen | Langzame solve (20 moves = 6+ sec) |
| Centrum-mapping voor layer 0 | Werkt na middelste laag rotaties | Complexere logica, extra edge case |
| rcuber two-phase solver | Optimale oplossingen (≤22 moves) | Grote dependency, init overhead |
| Graceful error handling | Geen crashes bij ongeldige states | Gebruiker ziet geen duidelijke feedback (alleen log) |

## 10. Toekomstige Uitbreidingen

### Custom solve-snelheid

Voeg een slider toe om animatie-duur (0.1s – 1.0s) aan te passen. Power users willen snellere solve, beginners langzamer.

### Solution preview

Toon de move-lijst (bijv. "R U R' U' ...") in een text-box voordat solve start. Gebruiker kan kopiëren voor analyse.

### Step-by-step solve

Pause-knop om solve te stoppen na elke move. "Next step" knop om door te gaan. Educatief voor beginners.

### Alternative solver algoritmes

- **Beginner method**: Layer-by-layer (40-60 moves, maar begrijpelijker)
- **CFOP/Fridrich**: Advanced method (meer moves dan Kociemba maar bekende patterns)

### Solve hints

In plaats van volledige solve: toon alleen de volgende 1-3 moves als hint. Gebruiker lost zelf verder op.

