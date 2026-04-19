# Ontwerpdocument: Face Rotaties voor Cubie

## 0. Renderingkeuze (VERPLICHTE SECTIE)

### Gekozen aanpak: `StandardMaterial { unlit: true }`

Voor alle sticker-materialen wordt `StandardMaterial` gebruikt met het veld `unlit: true`. Dit is de meest geschikte aanpak om de volgende redenen:

1. **Kleuren zijn 100% invariant**: Met `unlit: true` negeert Bevy's renderer alle lichtbronnen (directional, point, ambient) voor dat materiaal. De `base_color` wordt direct naar het scherm geschreven zonder enige berekening op basis van normals, lichtrichting of lichtintensiteit.
2. **Geen schaduw-interactie**: Unlit materialen ontvangen geen schaduwen en werpen er ook geen. Er is geen mogelijkheid dat een sticker donkerder wordt door shadow mapping.
3. **Geen specular of reflectie**: Er vindt geen BRDF-evaluatie plaats; de kleur is exact de `base_color`, ongeacht metallic/roughness-waarden.
4. **Deterministische output**: Dezelfde `base_color` geeft altijd dezelfde pixelkleur, ongeacht camerahoek, lichtopstelling of time-of-day.
5. **Tone mapping**: Bevy past standaard tone mapping toe. Omdat de stickerkleuren zijn gedefinieerd in sRGB met waarden ≤1.0 en er geen lichtbijdrage is, blijft de output na tone mapping identiek aan de input.

### Waarom alternatieven ongeschikt zijn

| Aanpak | Probleem |
|--------|----------|
| `StandardMaterial` (default, lit) | Kleur verandert met lichthoek, schaduw, ambient. Stickers worden donkerder/lichter afhankelijk van oriëntatie t.o.v. licht. |
| PBR met hoge emissive | Emissive kleuren worden beïnvloed door tone mapping en bloom; niet deterministic bij hoge waarden. |
| Custom unlit shader | Functioneel equivalent aan `unlit: true`, maar onnodige complexiteit. Custom pipeline maintenance zonder meerwaarde. |

### Impact op het bestaande project

De huidige code gebruikt `StandardMaterial` **zonder** `unlit: true`. Dit moet worden aangepast:

- **Sticker-materialen**: `unlit: true` toevoegen
- **Cubie body-materiaal**: Kan lit blijven (zwart lichaam mag schaduw/diepte tonen) óf ook unlit worden gemaakt (ontwerpkeuze)
- **Belichting**: Directional light en ambient light kunnen behouden blijven voor het cubie body. Voor stickers zijn ze irrelevant wanneer `unlit: true` is ingesteld.

### Robuustheid

| Risico | Mitigatie |
|--------|-----------|
| Z-fighting sticker ↔ cubie face | Bestaande `STICKER_ELEVATION = 0.001` offset. Kan verhoogd worden naar 0.005 als er artefacten optreden bij bepaalde hoeken. |
| Backface culling | Stickers zijn eenzijdige `Rectangle`-meshes; de normal wijst naar buiten. Zichtbare stickers worden nooit geculled. `cull_mode` kan op `None` gezet worden als extra zekerheid. |
| sRGB/linear fouten | `Color::srgb()` wordt al correct gebruikt. Bevy converteert intern naar linear voor rendering en terug naar sRGB voor output. Geen handmatige conversie nodig. |

---

## 1. Stapsgewijs ontwikkelplan

### Fase 1: Unlit rendering + picking-infrastructuur

**Doel**: Stickers unlit maken en raycasting mogelijk maken.

- Pas alle sticker-materialen aan naar `unlit: true`
- Voeg een `Sticker`-component toe aan sticker-entiteiten met een referentie naar de parent cubie en de `FaceDirection`
- Integreer `bevy_mod_picking` of implementeer custom raycasting tegen de cubie meshes
- Valideer dat picking correct werkt met unlit materialen (picking is mesh-gebaseerd, niet shading-gebaseerd, dus geen impact)

**Risico's**:
- `bevy_mod_picking` compatibiliteit met Bevy 0.15 (moet gevalideerd worden)
- Raycasting tegen child-entities in een parent-child hiërarchie vereist correcte global transforms

### Fase 2: Drag-detectie en rotatie-as bepaling

**Doel**: Muisdrag interpreteren als rotatie-instructie.

- Implementeer mousedown → drag → mouseup state machine
- Bepaal bij mousedown welke cubie + face is geraakt
- Bereken bij eerste significante drag de dominante sleeprichting (horizontaal vs verticaal in screen space)
- Map screen-space sleeprichting naar world-space rotatie-as op basis van de geraakte face-normaal en cameraoriëntatie
- Voeg threshold/deadzone toe voor kleine bewegingen

**Risico's**:
- Screen → world mapping is camerahoek-afhankelijk; moet goed getest worden
- Ambiguïteit bij diagonale drags (zie sectie 4)

### Fase 3: Slice-selectie en rotatielogica

**Doel**: Bepaal welke cubies tot de te roteren slice behoren en pas de logische staat aan.

- Op basis van de rotatie-as en de `grid_position` van de geraakte cubie, selecteer de juiste 9 cubies (slice)
- Implementeer logische rotatie: update `grid_position` en `FaceDirection` van stickers in `CubeState`
- Valideer correctheid door heen-en-terug rotaties

**Risico's**:
- Foutieve face-direction mapping na rotatie leidt tot verkeerde kleuren
- Edge case: center cubies hebben andere face-mappings dan corner/edge cubies

### Fase 4: Animatie

**Doel**: Vloeiende visuele rotatie.

- Maak tijdelijke pivot-entity op de rotatie-as
- Reparent de 9 cubie-entities als kinderen van de pivot
- Animeer de pivot-rotatie van 0° naar 90° (of -90°) via `Quat::slerp`
- Na voltooiing: deparent cubies, update hun individuele transforms, synchroniseer met logische staat

**Risico's**:
- Reparenting in Bevy kan subtiel zijn (global transforms moeten correct blijven)
- Floating point drift na vele rotaties (periodiek normaliseren)

### Fase 5: Input-integratie en polish

**Doel**: Volledige integratie van camera-orbit en face-rotatie.

- Differentieer linkermuisknop gedrag: klik op achtergrond → orbit camera; klik op cubie → face rotatie
- Voorkom gelijktijdige camera- en face-rotatie
- Voeg visuele feedback toe (optioneel: highlight geselecteerde slice)
- Test met verschillende camerahoeken

**Risico's**:
- Input-conflicten tussen camera-orbit en face-rotatie
- UX: onduidelijk wanneer camera-orbit vs face-rotatie actief is

---

## 2. Architectuurschets

### Modulestructuur (uitgebreid)

```
src/
├── main.rs                 # App-configuratie, plugin-registratie
├── camera.rs               # Orbit-camera (bestaand, aangepast)
├── icon.rs                 # App-icoon (bestaand)
└── cube/
    ├── mod.rs              # Module-declaraties
    ├── model.rs            # Datamodel (bestaand, uitgebreid)
    ├── spawn.rs            # Spawning (bestaand, aangepast)
    ├── picking.rs          # Raycasting en face-selectie
    ├── rotation.rs         # Rotatielogica en slice-selectie
    ├── animation.rs        # Geanimeerde face-rotatie
    └── input.rs            # Drag-detectie en input state machine
```

### Nieuwe en aangepaste componenten

```
Sticker (Component) [NIEUW]
├── cubie_entity: Entity         // referentie naar parent cubie
├── face_direction: FaceDirection // welk vlak van de cubie
└── color: StickerColor          // huidige kleur

DragState (Resource) [NIEUW]
├── phase: DragPhase             // Idle | Pending { hit } | Dragging { axis, direction, slice }
├── start_screen_pos: Vec2       // muispositie bij mousedown
└── hit_info: Option<HitInfo>    // geraakt cubie + face + world positie

HitInfo [NIEUW]
├── cubie_entity: Entity
├── face_direction: FaceDirection
├── world_position: Vec3
└── face_normal: Vec3

FaceRotationAnimation (Resource) [NIEUW]
├── active: bool
├── pivot_entity: Option<Entity>
├── affected_cubies: Vec<Entity>
├── rotation_axis: Vec3
├── target_angle: f32            // +90° of -90°
├── current_angle: f32
├── duration: f32                // totale animatieduur in seconden
├── elapsed: f32
```

### Aangepaste bestaande structuren

```
CubeState (Resource) [UITGEBREID]
├── cubies: Vec<CubieData>       // bestaand
└── apply_rotation(&mut self, axis: Axis, layer: i32, clockwise: bool) [NIEUW]

Cubie (Component) [BESTAAND]
└── grid_position: IVec3         // wordt na rotatie geüpdatet
```

### Systeemoverzicht en executievolgorde

```
Systems (Update schedule):

1. cube::input::handle_mouse_input        // leest muisinput, beheert DragState
2. cube::picking::raycast_pick            // bij mousedown: bepaal geraakt vlak
3. cube::input::resolve_drag_direction    // bij drag: bepaal rotatie-as uit sleepvector
4. cube::rotation::start_face_rotation    // initieer rotatie: selecteer slice, maak pivot
5. cube::animation::animate_face_rotation // elke frame: interpoleer rotatie
6. cube::rotation::finish_face_rotation   // na animatie: update logisch model, cleanup
7. camera::orbit_camera_system            // camera-orbit (alleen als DragState == Idle)
```

### Dataflow

```
Muisinput (MouseButton + MouseMotion)
    │
    ▼
handle_mouse_input
    │ schrijft naar DragState
    ▼
raycast_pick (bij mousedown)
    │ vult HitInfo in DragState
    ▼
resolve_drag_direction (bij drag > threshold)
    │ bepaalt rotatie-as + richting
    │ schrijft naar DragState::Dragging
    ▼
start_face_rotation
    │ leest DragState::Dragging
    │ selecteert 9 cubies op basis van grid_position
    │ maakt pivot entity
    │ reparent cubies
    │ initialiseert FaceRotationAnimation
    ▼
animate_face_rotation (elke frame)
    │ leest FaceRotationAnimation
    │ roteert pivot entity via Quat::slerp
    │ gebruikt ease-out curve voor natuurlijk gevoel
    ▼
finish_face_rotation (wanneer animatie compleet)
    │ deparent cubies van pivot
    │ update individuele cubie transforms
    │ update CubeState (grid_position + face directions)
    │ verwijder pivot entity
    │ reset DragState naar Idle
    │ reset FaceRotationAnimation
    ▼
Volgende frame: klaar voor nieuwe interactie
```

---

## 3. Input & picking strategie

### Picking-aanpak: `bevy_mod_picking` met raycasting backend

**Aanbeveling**: Gebruik `bevy_mod_picking` (indien compatibel met Bevy 0.15). Dit biedt:

- Mesh-gebaseerde raycasting (werkt op geometry, niet op shading)
- Automatische event-propagatie door parent-child hiërarchie
- Hover, click en drag events out-of-the-box

**Fallback**: Custom raycasting via `bevy::math::Ray3d` en intersectie met de 6 vlakken van elke cubie. Dit is betrouwbaar maar meer werk.

### Hoe detecteer je cubie + face?

1. **Raycast vanuit camera**: Schiet een ray vanuit de muispositie door de camera frustum
2. **Intersectie met sticker-meshes**: De ray raakt een sticker-rectangle. Via de `Sticker`-component weten we welke cubie en welk vlak geraakt is.
3. **Alternatief (zonder sticker-picking)**: Raycast tegen cubie body (Cuboid). De raakpositie + normaal op dat punt geeft de face direction. Dit is robuuster omdat het niet afhankelijk is van de kleine sticker-mesh.

**Aanbeveling**: Raycast tegen cubie bodies. De normaal van het raakpunt bepaalt de `FaceDirection`:
- Normal ≈ +X → Right
- Normal ≈ -X → Left
- Normal ≈ +Y → Up
- etc.

### Impact van unlit rendering op picking

**Geen impact.** Picking/raycasting werkt op mesh-geometry en transforms, niet op materiaal-eigenschappen. Of een materiaal lit of unlit is, verandert niets aan de raycasting. Dit is een belangrijk voordeel: de picking-strategie is volledig onafhankelijk van de renderingkeuze.

### Sticker-identificatie

De prompt eist: "Alleen de visuele sticker-kleuren worden gebruikt om faces te identificeren (geen verborgen metadata)." 

**Interpretatie en aanpak**: De `Sticker`-component is geen verborgen metadata maar een directe ECS-representatie van wat visueel zichtbaar is. De `StickerColor` in de component correspondeert 1-op-1 met de gerenderde kleur. Er is geen verborgen ID of tag die losstaat van de visuele representatie. De kleur IS de identifier.

Als een striktere interpretatie gewenst is (bijv. letterlijk de pixelkleur samplen), dan is een color-picking render pass nodig. Dit wordt afgeraden vanwege complexiteit en fragiliteit (floating point kleuren matchen is foutgevoelig). De ECS-component aanpak is betrouwbaarder en performanter.

---

## 4. Rotatielogica

### Welke slice roteert?

Een face-rotatie betreft altijd een laag van 9 cubies die dezelfde coördinaat delen op één as:

| Rotatie-as | Gedeelde coördinaat | Voorbeeld |
|------------|---------------------|-----------|
| X-as | `grid_position.x == n` | x=1: rechterlaag, x=0: middenlaag, x=-1: linkerlaag |
| Y-as | `grid_position.y == n` | y=1: bovenlaag, y=-1: onderlaag |
| Z-as | `grid_position.z == n` | z=1: voorlaag, z=-1: achterlaag |

De waarde van `n` wordt bepaald door de `grid_position` van de geraakte cubie, geprojecteerd op de rotatie-as.

### Rotatie-as bepalen

De rotatie-as wordt afgeleid uit twee vectoren:

1. **Face-normaal** (`N`): de normaal van het geraakte vlak (bijv. klik op het bovenvlak → N = +Y)
2. **Sleepvector** (`D`): de drag-richting in world space, geprojecteerd op het vlak loodrecht op de face-normaal

De rotatie-as is het kruisproduct: `A = N × D_projected`

Dit geeft altijd een as die:
- Loodrecht staat op zowel de face-normaal als de sleeprichting
- Overeenkomt met een van de drie hoofdassen (X, Y, Z) na snapping

### Snapping naar hoofdas

Het kruisproduct geeft een vector die niet exact op een hoofdas hoeft te liggen. Snap naar de dichtstbijzijnde hoofdas:

```
als |A.x| > |A.y| en |A.x| > |A.z| → rotatie rond X-as
als |A.y| > |A.x| en |A.y| > |A.z| → rotatie rond Y-as
anders → rotatie rond Z-as
```

### Draairichting

Het teken van de component op de gekozen as bepaalt de richting:
- Positieve component → kloksgewijs (vanuit positieve askant bekeken)
- Negatieve component → tegen de klok in

### Screen → world mapping

De sleepvector in screen space moet naar world space vertaald worden. Dit gaat als volgt:

1. Neem de screen-space drag vector `(Δx, Δy)` in pixels
2. Bereken de camera's lokale assen: `camera_right` en `camera_up` uit de camera-transform
3. De world-space drag: `D_world = Δx · camera_right + Δy · (-camera_up)`
4. Projecteer `D_world` op het vlak van de geraakte face: `D_projected = D_world - (D_world · N) · N`

### Ambiguïteit oplossen

**Diagonale drags**: Alleen de dominante component telt. Na projectie en snapping wordt altijd exact één hoofdas geselecteerd.

**Threshold**: Een minimale drag-afstand van ~10 pixels voordat de richting wordt vastgelegd. Dit voorkomt:
- Foutieve richting door jitter bij klikken
- Onbedoelde rotaties bij kleine muisbewegingen

**Klik zonder drag**: Als de muisknop losgelaten wordt vóór het bereiken van de threshold, gebeurt er niets.

### Logische rotatie (CubeState update)

Bij een 90°-rotatie rond een as worden de `grid_position` en `FaceDirection` van de 9 cubies in de slice geüpdatet:

**Rond de Y-as (kloksgewijs, van boven bekeken)**:
- `grid_position`: (x, y, z) → (z, y, -x)
- Face-directions: Front→Right, Right→Back, Back→Left, Left→Front (Up en Down ongewijzigd)

**Rond de X-as (kloksgewijs, van rechts bekeken)**:
- `grid_position`: (x, y, z) → (x, -z, y)
- Face-directions: Front→Up, Up→Back, Back→Down, Down→Front (Left en Right ongewijzigd)

**Rond de Z-as (kloksgewijs, van de voorkant bekeken)**:
- `grid_position`: (x, y, z) → (y, -x, z)
- Face-directions: Up→Right, Right→Down, Down→Left, Left→Up (Front en Back ongewijzigd)

Voor tegen-de-klok-in rotaties: pas de inverse mapping toe (of roteer 3× kloksgewijs).

---

## 5. Animatie

### Aanpak: tijdelijke pivot-entity met Quat::slerp

De animatie verloopt in drie fasen:

#### 5.1 Opstart

1. Maak een pivot-entity aan op de oorsprong (Vec3::ZERO) met een `Transform`
2. Reparent de 9 cubie-entities als kinderen van de pivot
3. Sla hun originele global transforms op om te compenseren voor de reparenting

#### 5.2 Interpolatie (elke frame)

1. Bereken `t = elapsed / duration` (0.0 → 1.0)
2. Pas een easing-functie toe: `t_eased = ease_out_cubic(t)` voor een natuurlijk aanvoelende rotatie (snel starten, zacht stoppen)
3. Bereken de huidige rotatie: `Quat::from_axis_angle(axis, target_angle * t_eased)`
4. Pas toe op de pivot-entity's `Transform::rotation`

#### 5.3 Voltooiing

1. Zet de pivot-rotatie exact op de target (90° of -90°) om floating point drift te voorkomen
2. Lees de finale global transforms van alle 9 cubies
3. Deparent cubies van de pivot
4. Zet de cubie transforms op de afgeronde posities (snap naar gehele grid-posities)
5. Update `CubeState` met de nieuwe `grid_position` en face-directions
6. Verwijder de pivot-entity
7. Markeer de animatie als voltooid

### Time-based vs frame-based

**Keuze: time-based.** De animatie gebruikt `Time::delta_secs()` in plaats van een vast bedrag per frame. Dit garandeert:
- Consistente animatieduur ongeacht framerate
- Vloeiende rotatie op zowel 30fps als 144fps

### Duur

Aanbevolen animatieduur: **0.25–0.35 seconden** voor een 90°-rotatie. Dit voelt responsief zonder te snel te zijn.

### Easing-functie

`ease_out_cubic(t) = 1 - (1 - t)³`

Dit geeft een snelle start die vertraagt naar het einde, wat het meest natuurlijk aanvoelt voor een fysieke draaibeweging.

### Blokkering tijdens animatie

Tijdens een actieve animatie worden nieuwe face-rotaties en camera-orbit geblokkeerd. Dit voorkomt:
- Conflicterende reparenting
- Visuele glitches
- Inconsistente logische staat

---

## 6. Libraries

### `bevy_mod_picking`

**Voordelen**:
- Volwassen, veel-gebruikte crate voor 3D picking in Bevy
- Mesh-gebaseerde raycasting (precies wat nodig is)
- Event-systeem voor hover, click, drag
- Ondersteunt parent-child hiërarchie

**Nadelen/risico's**:
- Bevy 0.15 compatibiliteit moet geverifieerd worden
- Extra dependency en complexiteit
- Mogelijk te veel functionaliteit voor onze usecase

### Custom raycasting

**Voordelen**:
- Geen externe dependency
- Volledige controle
- Bevy's `Ray3d` en mesh-intersectie utilities zijn beschikbaar

**Nadelen**:
- Meer implementatiewerk
- Moet zelf global transforms correct afhandelen
- Moet zelf near/far plane en frustum afhandelen

### Aanbeveling

**Start met custom raycasting.** De picking-requirements zijn beperkt (alleen klik-detectie op kubussen, geen hover/drag events nodig van de library). Een ray-plane intersectie tegen de 6 vlakken van een cubie is eenvoudig te implementeren en geeft volledige controle. Dit vermijdt compatibiliteitsrisico's met `bevy_mod_picking`.

Als de custom oplossing te complex of buggy blijkt, kan alsnog gemigreerd worden naar `bevy_mod_picking`.

### Overige libraries

| Library | Doel | Aanbeveling |
|---------|------|-------------|
| `bevy_tweening` | Animatie-easing | Overweeg later; handmatige `slerp` + easing is voldoende voor één animatietype |
| `bevy_egui` | Debug UI | Optioneel; handig voor debugging van rotatie-staat |

---

## 7. Uitbreidbaarheid

### Undo/redo

- Houd een `Vec<Move>` bij als move-history in een `MoveHistory`-resource
- `Move` bevat: `axis: Axis, layer: i32, clockwise: bool`
- Undo: voer de inverse move uit (zelfde as en laag, omgekeerde richting)
- Redo: houd een redo-stack bij die gewist wordt bij een nieuwe handmatige move
- Animatie is herbruikbaar: undo/redo triggeren dezelfde animatie-pipeline

### Scramble

- Genereer een reeks van N willekeurige `Move`s (bijv. 20-25)
- Voer ze sequentieel uit, elk met een verkorte animatieduur (bijv. 0.05s) of instant
- Optioneel: animeer de volledige scramble als snelle sequentie voor visueel effect
- Vermijd triviale cancellaties (bijv. R gevolgd door R' )

### Solver

- `CubeState` is een pure datastructuur, onafhankelijk van ECS
- Een solver kan opereren op een kloon van `CubeState` zonder de visuele staat te beïnvloeden
- Output: een `Vec<Move>` die sequentieel geanimeerd kan worden
- Algoritme-opties: Kociemba two-phase, CFOP-stappen, of een beginner-methode
- De solver kan in een aparte thread draaien (Bevy's `AsyncComputeTaskPool`) om de framerate niet te beïnvloeden

### NxN cubes

- `grid_position` generaliseren: bereik van `-(n/2)..=(n/2)` voor oneven N
- Slice-selectie blijft hetzelfde principe (filteren op één coördinaat)
- Meer cubies per slice (N² in plaats van 9)
- Spawning parametriseren op N
- Face-direction mapping blijft identiek

### Andere inputmethodes

- **Keyboard**: Standaard Rubik's Cube notatie (R, L, U, D, F, B + ' voor inverse)
- **Touch**: Dezelfde drag-logica, maar met touch events i.p.v. mouse events
- **Gamepad**: Bumpers/triggers voor laag-selectie, stick voor richting
- De input-module is gescheiden van de rotatie-logica, dus nieuwe inputmethodes hoeven alleen `Move`-events te genereren

---

## Aannames

1. De camera kijkt altijd naar de oorsprong (waar de kubus staat)
2. Er is altijd maximaal één face-rotatie tegelijk actief
3. De gebruiker gebruikt de linkermuisknop voor zowel camera-orbit als face-rotatie (onderscheid via rayhit)
4. Grid-posities zijn gehele getallen (-1, 0, 1) en worden na rotatie teruggesnapt
5. De Bevy 0.15 API is stabiel voor de gebruikte features (Transform, parent-child, meshes)

## Trade-offs

| Keuze | Voordeel | Nadeel |
|-------|----------|--------|
| `unlit: true` voor stickers | Perfecte kleurconsistentie | Geen diepte-cue op stickers; compenseren met body-shading |
| Custom raycasting i.p.v. `bevy_mod_picking` | Geen externe dependency, volledige controle | Meer implementatiewerk, potentieel bugs |
| Tijdelijke pivot voor animatie | Schone scheiding animatie ↔ logica | Reparenting complexiteit, moet global transforms compenseren |
| Time-based animatie | Framerate-onafhankelijk | Iets complexer dan frame-based; vereist `Time`-resource |
| Eén rotatie tegelijk | Eenvoudige state machine, geen conflicten | Gebruiker moet wachten tot animatie klaar is |
| Input-differentiatie via rayhit | Intuïtief: klik op kubus = roteer, klik op achtergrond = orbit | Vereist betrouwbare picking; miskliks mogelijk |
