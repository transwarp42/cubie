Je bent een senior Rust graphics programmeur en game engine architect, gespecialiseerd in Bevy en ECS-architecturen.

Je denkt strikt in ECS (data + systems) en vermijdt objectgeoriënteerde patronen.

---

## Context

We hebben een Rust applicatie genaamd `Cubie`:

- Rendert een 3x3 Rubik’s Cube in 3D
- Ondersteunt rotatie van:
    - een zijde
    - de volledige cube
- Input gebeurt via muisinteractie
- Architectuur is ECS-gebaseerd (Bevy)

---

## Doel

Ontwerp een robuuste undo/redo architectuur voor `Cubie`.

---

## Functionele eisen

### Interne stacks

- Undo stack → bevat uitgevoerde acties
- Redo stack → bevat ongedaan gemaakte acties

Flow:

1. Nieuwe actie → push naar undo stack
2. Undo → verplaats actie naar redo stack
3. Redo → verplaats actie terug naar undo stack
4. Nieuwe actie na undo → redo stack wordt geleegd

---

### Definitie van een “actie” (VERPLICHT)

Een actie moet:

- deterministisch zijn
- omkeerbaar zijn (inverse beschikbaar)
- losstaan van rendering
- geschikt zijn voor ECS (data-first)

Voorbeelden:
- Rotatie van één zijde
- Rotatie van de volledige cube

---

### UI gedrag

- Undo button links van `Labels`
- Redo button links van Undo
- Buttons zijn alleen actief indien stack niet leeg is

---

## Technische randvoorwaarden

- Programmeertaal: Rust
- Engine: Bevy
- ECS-first architectuur
- Strikte scheiding:
    - Input
    - Logica (state mutations)
    - Rendering

---

## Belangrijke ontwerpkeuzes (MOETEN behandeld worden)

### 1. State management strategie

Vergelijk en kies:

- Command-based (actions + inverse)
- Snapshot-based
- Hybrid

Leg uit:
- waarom jouw keuze het beste past bij Bevy/ECS
- performance implicaties
- geheugenimpact

---

### 2. Animatie vs state

- Hoe ga je om met:
    - geanimeerde rotaties
    - undo tijdens animatie
- Is state direct of deferred?

---

### 3. ECS integratie

- Hoe worden acties opgeslagen?
    - Resource?
    - Events?
- Hoe blijven systems stateless?
- Hoe voorkom je tight coupling?

---

### 4. Edge cases

Behandel expliciet:

- Nieuwe actie tijdens undo-state
- Undo tijdens lopende animatie
- Snelle input (race conditions)
- Determinisme van acties

---

## Wat ik verwacht (LEVER GEEN CODE)

🚫 Geen code  
🚫 Geen pseudo-code  
🚫 Geen Rust structs of signatures

---

## Output structuur (VERPLICHT)

### 0. Renderingkeuze (VERPLICHTE SECTIE)

- Kies expliciet hoe je unlit rendering implementeert
- Leg uit waarom dit voldoet aan de harde kleur-eis
- Leg uit waarom alternatieven (zoals PBR) ongeschikt zijn

---

### 1. Stapsgewijs ontwikkelplan

- Fases
- Risico’s per fase
- Validatiestrategie

---

### 2. Architectuurschets

- Modulestructuur
- Components
- Resources
- Systems

Dataflow (verplicht):

input → picking → selectie → actie → undo/redo → animatie → render

---

### 3. Undo/Redo model

- Representatie van acties
- Opslag in stacks
- Inverse logica
- Integratie met ECS

---

### 4. Libraries / tooling

Bijv.:

- `bevy_mod_picking`
- Custom oplossingen

Per optie:
- Voordelen
- Nadelen

---

## Output eisen

- Eén gestructureerd Markdown document
- Duidelijke koppen
- Technisch diepgaand
- Geen code
- Aannames expliciet
- Trade-offs benoemen
- Schrijf in het Nederlands