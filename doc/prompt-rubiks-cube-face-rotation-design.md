Je bent een ervaren Rust graphics programmeur en game engine architect, gespecialiseerd in Bevy en ECS-architecturen.

## Context

We hebben een Rust applicatie genaamd `Cubie`:

- Rendert een 3x3 Rubik’s Cube in 3D
- Ondersteunt momenteel alleen rotatie van de volledige cube via muisinput

## KRITISCHE VISUELE EIS (HARD REQUIREMENT)

De kleuren van de stickers zijn functioneel en moeten **altijd exact correct zichtbaar zijn**.

### VERPLICHT

- Sticker-kleuren moeten:
  - Altijd identiek zijn aan de gedefinieerde basiskleur
  - Onafhankelijk zijn van licht, schaduw en kijkhoek
  - Nooit donkerder, lichter of zwart worden

### VERBODEN

De volgende technieken mogen **NIET** gebruikt worden voor stickers:

- ❌ PBR lighting (StandardMaterial met lighting actief)
- ❌ Schaduwen (shadow casting / receiving)
- ❌ Normals-gebaseerde shading
- ❌ Ambient light afhankelijkheid
- ❌ Specular highlights
- ❌ Tone mapping die kleur beïnvloedt

### VERPLICHTE OPLOSSING

Je moet expliciet één van de volgende kiezen en motiveren:

- ✅ `StandardMaterial { unlit: true }`
- ✅ Custom unlit shader
- ✅ Alternatieve flat-color rendering aanpak zonder lighting

Als je een andere aanpak kiest, moet je bewijzen dat:

- kleuren 100% invariant blijven
- er geen enkele afhankelijkheid is van licht of normals

### EXTRA ROBUUSTHEID

Je ontwerp moet expliciet rekening houden met:

- Geen z-fighting tussen sticker en cubie face
- Geen backface-culling artefacten op zichtbare stickers
- Geen kleurvervorming door sRGB/linear fouten
- Deterministische rendering (zelfde kleur = altijd zelfde output)

## Doel

Voeg ondersteuning toe voor het roteren van individuele zijdes (face rotations) via muisinteractie.

## Functionele eisen

### Interactiegedrag

1. De gebruiker klikt op een zichtbaar vlak (face) van een cubelet
2. De gebruiker sleept (drag) met de muis
3. De initiële sleepvector bepaalt:

   - De rotatie-as
   - De draairichting

4. De rotatie wordt vloeiend geanimeerd (geen instant snapping)
5. Alleen de visuele sticker-kleuren worden gebruikt om faces te identificeren (geen verborgen metadata)

## BELANGRIJK: implicatie van bovenstaande eis

Omdat kleuren leidend zijn:

- De picking/identificatie moet betrouwbaar blijven zonder lighting cues
- Contrast tussen stickers moet puur uit kleur komen
- Geen reliance op shading voor visuele interpretatie

## Input → rotatie mapping (screen space → world space)

- Horizontale muisbeweging → rotatie rond een verticale as
- Verticale muisbeweging → rotatie rond een horizontale as

## Richtingsinterpretatie

- Sleeprichting bepaalt draairichting relatief aan de camera
- Mapping naar world space moet expliciet gedefinieerd worden
- Gedrag moet consistent blijven bij elke camerahoek

## Edge cases

- Diagonale bewegingen → dominante as bepalen
- Kleine bewegingen → threshold / deadzone
- Klik zonder drag → geen rotatie

## Technische randvoorwaarden

- Programmeertaal: Rust
- Engine: Bevy
- ECS-gebaseerde architectuur
- Scheiding tussen:
  - Rendering
  - Input
  - Logica

## Wat ik verwacht (LEVER GEEN CODE)

### 0. Renderingkeuze (VERPLICHTE SECTIE)

- Kies expliciet hoe je unlit rendering implementeert
- Leg uit waarom dit voldoet aan de harde kleur-eis
- Benoem waarom alternatieven (bijv. PBR) ongeschikt zijn

### 1. Stapsgewijs ontwikkelplan

- Fasering
- Risico’s per stap

### 2. Architectuurschets

- Modulestructuur
- Components, resources, systems
- Dataflow:
  input → picking → selectie → rotatie → animatie → render

### 3. Input & picking strategie

- Hoe detecteer je cubelet + face?
- Raycasting / picking
- Hoe beïnvloedt unlit rendering deze keuze?

### 4. Rotatielogica

- Welke slice roteert
- Rotatie-as
- Draairichting
- Screen → world mapping
- Ambiguïteit oplossen

### 5. Animatie

- Vloeiende rotatie
- Time-based vs frame-based
- Bevy patterns (lerp, tweening, etc.)

### 6. Libraries

- Bijv.:
  - `bevy_mod_picking`
  - Custom oplossing
- Trade-offs

### 7. Uitbreidbaarheid

- Undo/redo
- Scramble
- Solver
- NxN cubes
- Andere inputmethodes

## Output formaat

- Eén gestructureerd Markdown document
- Duidelijke koppen
- Technisch en concreet
- Geen code
- Aannames expliciet
- Trade-offs benoemen
- Schrijf in het Nederlands