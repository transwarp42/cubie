Je bent een senior Rust graphics programmeur en game engine architect, gespecialiseerd in Bevy en ECS-architecturen.

Je redeneert strikt volgens ECS-principes:
- Data (Components) is gescheiden van gedrag (Systems)
- Systems zijn stateless
- Communicatie verloopt via Events en Resources
- Vermijd objectgeoriënteerde patronen volledig

---

## Context

We hebben een Rust applicatie genaamd `Cubie`:

- Rendert een 3x3 Rubik’s Cube in 3D
- Ondersteunt rotatie van:
  - individuele zijden
  - de volledige cube
- Bevat undo/redo functionaliteit (actie-gebaseerd)
- Input via muisinteractie (picking + drag)
- Architectuur: Bevy ECS

---

## Doel

Ontwerp een robuuste, uitbreidbare en ECS-consistente scramble architectuur.

⚠️ Focus op architectuur en ontwerpkeuzes — NIET op implementatie.

---

## Functionele eisen

### Definitie van een scramble (VERPLICHT)

Er zijn twee concepten:

#### 1. Random-move scramble (baseline)
- 20–25 willekeurige moves
- Moves: U, D, L, R, F, B
- Rotaties: 90°, 180°, 270°
- Geen directe herhaling van hetzelfde vlak

#### 2. Random-state scramble (VERPLICHTE methode)
- Genereer eerst een geldige, willekeurige cube state
- Bereken daarna een reeks moves die exact naar deze state leidt
- Dit moet de primaire aanpak zijn

Referentie:
- https://www.worldcubeassociation.org/regulations/scrambles

---

### UI gedrag

- "Scramble" knop links van `Labels` en rechts van `Undo`
- Klik → bevestigingsdialog
- Na bevestiging:
  - nieuwe scramble wordt gegenereerd
  - cube gaat naar nieuwe staat via animatie (niet instant)

### Undo/Redo gedrag (BELANGRIJK)

- De scramble wordt NIET opgeslagen als reeks undoable acties
- Tijdens de scramble animatie worden GEEN acties naar de undo stack geschreven
- Na voltooiing van de scramble:
  - wordt de huidige cube state de nieuwe baseline
  - undo stack = leeg
  - redo stack = leeg

Interpretatie:
- Scramble is een **state reset**, geen user action
- De history start NA de scramble

---

## Technische randvoorwaarden

- Rust + Bevy
- ECS-first architectuur
- Strikte scheiding:
  - Input
  - State mutations
  - Rendering
- Systems mogen geen verborgen state bevatten

---

## Verplichte ontwerpbeslissingen

### 1. State management strategie

Vergelijk en kies:

- Command-based (actions + inverse)
- Snapshot-based
- Hybrid

Beantwoord:

- Waarom past dit bij ECS/Bevy?
- Wat zijn de performance implicaties?
- Wat is de geheugenimpact?
- Hoe integreert dit met undo/redo + scramble reset?

---

### 2. ECS integratie

Behandel expliciet:

- Waar worden acties opgeslagen?
  - Events?
  - Resources?
  - Anders?
- Hoe blijft elke system stateless?
- Hoe voorkom je tight coupling tussen systems?
- Hoe wordt de scramble pipeline opgesplitst in systems?

---

### 3. Dataflow (VERPLICHT)

Werk deze flow concreet uit:

input → picking → selectie → actie → confirmation → scramble generation → state calculation → animation → rendering

Voor ELKE stap:
- welk system is verantwoordelijk
- welke data wordt gelezen/geschreven

---

## Wat ik verwacht (STRIKT)

🚫 GEEN code  
🚫 GEEN pseudocode  
🚫 GEEN Rust types/signatures

Alleen architectuur, modellen en ontwerpkeuzes.

---

## Output structuur (VERPLICHT)

### 0. Renderingkeuze (VERPLICHT)

- Kies expliciet een unlit rendering aanpak
- Leg uit hoe dit consistente kubuskleuren garandeert
- Leg uit waarom PBR ongeschikt is in deze context

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

---

### 3. Scramble model

- Representatie van cube state
- Representatie van moves
- Hoe random-state wordt gegenereerd
- Hoe moves worden afgeleid uit state
- Integratie met ECS en animation pipeline

---

### 4. Libraries / tooling

Bijv.:

- `bevy_mod_picking`
- Eigen implementaties

Per optie:
- Voordelen
- Nadelen
- Wanneer kiezen

---

## Output eisen

- Eén gestructureerd Markdown document
- Duidelijke koppen
- Technisch diepgaand
- Alle aannames expliciet
- Trade-offs benoemen
- Schrijf in het Nederlands