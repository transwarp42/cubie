Je bent een senior Rust graphics programmeur en game-engine-architect, gespecialiseerd in Bevy en ECS-architecturen.

Je redeneert strikt volgens ECS-principes:
- Data (components) is strikt gescheiden van gedrag (systems)
- Systems zijn stateless
- Communicatie verloopt via events en resources
- Geen objectgeoriënteerde patronen
- Geen verborgen of impliciete state
- Elke ontwerpkeuze wordt expliciet gemotiveerd vanuit ECS/Bevy

---

# Context

We hebben een Rust-applicatie genaamd `Cubie`:

- Rendert een 3x3 Rubik's Cube in 3D
- Ondersteunt rotatie van individuele zijden en de volledige cube
- Bevat undo/redo (actie-gebaseerd via `ActionHistory`)
- Ondersteunt random-state scramble via `ScrambleQueue` (reeds geïmplementeerd)
- Input via muisinteractie (picking + drag)
- Architectuur: Bevy ECS

## Bestaande mechanismen die de solver **volledig hergebruikt**

| Concept | Bestaande implementatie |
|---|---|
| Logische cube state | `CubeState` resource (cubies met grid_position en stickers) |
| Move type | `CubeMove` (axis, layer, clockwise) |
| Move-queue patroon | `ScrambleQueue` (VecDeque<CubeMove> + status state machine) |
| Move-animatie | `FaceRotationAnimation` resource + bestaande animation pipeline |
| Acties vastleggen | `ActionHistory` (undo/redo stacks) |
| Solver library | `rcuber` crate met `Min2PhaseSolver` — **al als dependency aanwezig** |
| rcuber → CubeMove mapping | `rcuber_move_to_cube_moves()` functie — **al geïmplementeerd in scramble.rs** |
| ActionOrigin | `ActionOrigin` enum — enkel een `Solve`-variant toevoegen |

⚠️ De solver introduceert **geen nieuw state-mechanisme**. De enige nieuwe logica is de **bidirectionele mapping** tussen `CubeState` en het rcuber cube-formaat.

---

# Doel

Ontwerp een minimale, ECS-consistente solver-architectuur die maximaal leunt op de bestaande opzet van `Cubie`.

De solver werkt uitsluitend op een reeds gescramblede cube.

⚠️ Focus uitsluitend op architectuur en ontwerpkeuzes — niet op implementatie.

---

# Scope (strikt)

IN SCOPE:
- Mappen van `CubeState` → rcuber cube-formaat (solver-input)
- Mappen van rcuber solver-output → `VecDeque<CubeMove>` (via bestaande `rcuber_move_to_cube_moves`)
- Een `SolveQueue` resource als directe tegenhanger van `ScrambleQueue`
- Integratie met de bestaande animation pipeline
- Integratie met `ActionHistory`
- Een `Solve`-knop in de UI

OUT OF SCOPE:
- Genereren van scrambles
- Concrete code of API's
- Renderingkeuzes (reeds vastgelegd in de codebase)
- Nieuwe state-mechanismen

---

# Functionele eisen

## Solver definitie

De solver:
- leest de bestaande `CubeState` resource
- mapt deze naar het rcuber cube-formaat
- roept `Min2PhaseSolver` aan (al aanwezig als dependency)
- ontvangt een `Vec<RcuberMove>`
- converteert deze naar `VecDeque<CubeMove>` via de bestaande `rcuber_move_to_cube_moves`
- vult een `SolveQueue` — het directe equivalent van `ScrambleQueue`

De solver is conceptueel een pure transformatie:

`CubeState` → `VecDeque<CubeMove>`

De enige niet-triviale stap hierin is de **mapping van `CubeState` naar rcuber-formaat**. Dit is de kern van het ontwerp.

---

# UI gedrag

Een `Solve`-knop (links van `Scramble`):

Bij activatie:

1. De actuele `CubeState` wordt gelezen en direct gemapped naar rcuber cube-formaat — dit vereist geen animatie of extra state-capture
2. Als visueel effect wordt elke zijde één voor één geanimeerd naar voren gedraaid, alsof een virtuele camera elke kant "fotografeert" — dit is puur cosmetisch en heeft geen invloed op de state-capture of de solver-input
3. `Min2PhaseSolver` berekent de oplossing
4. De oplossing wordt via `rcuber_move_to_cube_moves` omgezet naar `VecDeque<CubeMove>`
5. De `SolveQueue` wordt gevuld en geactiveerd
6. Moves worden een voor een geanimeerd uitgevoerd via dezelfde pipeline als scramble

⚠️ Het "fotografeer"-effect (stap 2) is uitsluitend een visuele presentatiekeuze. De werkelijke solver-input wordt rechtstreeks afgeleid uit `CubeState` — zonder tussenliggende camera, rendering of state-opslag.

Er vindt geen scramble-generatie plaats. Er is geen bevestigingsdialoog nodig (de solver lost enkel op, reset niets).

---

# Technische randvoorwaarden

- Rust + Bevy
- ECS-first architectuur
- Maximaal hergebruik van bestaande systems, resources en patronen
- Systems zijn stateless
- Geen side-effects buiten ECS-data
- De implementatie moet zo eenvoudig mogelijk zijn — elke toegevoegde complexiteit vereist motivatie

---

# Verplichte ontwerpbeslissingen

## 1. Mapping: CubeState → rcuber formaat

Dit is de kernvraag van het ontwerp. Beantwoord:

- Hoe wordt de huidige `CubeState` (cubies met grid_position en stickers) omgezet naar het rcuber cube-formaat?
- Welke aannames worden gemaakt over de oriëntatie van de cube (center-sticker kleuren als referentie)?
- Hoe worden de zes zijden in de juiste volgorde uitgelezen?
- Wat zijn de randgevallen bij het mappen (bijv. rotatie van de gehele cube, afwijkende oriëntatie)?
- Waarom is deze mapping robuust genoeg voor een langdurig onderhouden codebase?

## 2. SolveQueue als hergebruik van ScrambleQueue-patroon

Beantwoord:

- Hoe verhoudt `SolveQueue` zich exact tot `ScrambleQueue`?
- Welke systems van de scramble-pipeline kunnen direct worden hergebruikt?
- Welke systems moeten worden aangepast of gedupliceerd?
- Hoe wordt voorkomen dat solve en scramble tegelijk actief zijn?

## 3. Integratie met ActionHistory

Beantwoord:

- Worden solve-moves opgeslagen in `ActionHistory` zodat undo werkt na een solve?
- Hoe verhoudt dit zich tot het gedrag van scramble (die history wist)?
- Wat is de juiste keuze en waarom?

## 4. ECS integratie

Behandel expliciet:

- Waar leeft de `SolveQueue`?
- Welke events worden gebruikt?
- Hoe blijven systems stateless?
- Hoe wordt tight coupling vermeden?

---

## 5. Dataflow (verplicht, volledig uitwerken)

Werk deze flow uit:

`Solve`-knop ingedrukt  
→ CubeState uitlezen + direct mappen naar rcuber-formaat  
→ Visueel "fotografeer"-effect: zijden één voor één naar voren animeren (puur cosmetisch)  
→ Min2PhaseSolver aanroepen  
→ rcuber output → VecDeque<CubeMove>  
→ SolveQueue vullen  
→ SolveQueue verwerken (per move)  
→ FaceRotationAnimation starten  
→ AnimationFinished → volgende move  
→ SolveQueue leeg → idle

Voor elke stap:

- verantwoordelijk system
- gelezen data
- geschreven data
- type communicatie (event/resource/component)
- motivatie waarom deze stap hier zit

---

# Wat ik NIET wil zien

🚫 Code  
🚫 Pseudocode  
🚫 Rust types  
🚫 Function signatures  
🚫 Implementatiedetails  
🚫 Renderingkeuzes (buiten scope)  
🚫 Nieuwe state-mechanismen die de bestaande dupliceren

---

# Verplichte outputstructuur

## 1. Stapsgewijs ontwikkelplan

- Fases
- Risico's per fase (focus op de mapping-stap)
- Validatiestrategie
- Wat moet vroeg vastliggen vs. later kan evolueren

---

## 2. Architectuurschets

- Welke bestaande modules worden uitgebreid vs. welke nieuwe module(s) worden toegevoegd
- Nieuwe resources en hun verantwoordelijkheid
- Nieuwe systems en hun verantwoordelijkheid
- Grenzen tussen subsystemen

---

## 3. Mapping-model: CubeState ↔ rcuber

- Hoe wordt de actuele cube state uitgelezen uit `CubeState`
- Hoe worden facelets bepaald per zijde (volgorde, oriëntatie)
- Hoe wordt de rcuber output teruggezet naar `CubeMove` (via bestaande functie)
- Randgevallen en aannames

---

## 4. SolveQueue ontwerp

- Structuur (analoog aan `ScrambleQueue`)
- Status state machine — inclusief een `Scanning`-fase voor het visuele "fotografeer"-effect vóór de eigenlijke solve-animatie
- Hoe het visuele scan-effect wordt geïmplementeerd: welke zijden, in welke volgorde, welke animatie-techniek (hergebruik van bestaande rotatie-animatie?)
- Welke bestaande scramble-systems direct herbruikbaar zijn
- Welke aanpassingen nodig zijn

---

## 5. ActionHistory integratie

- Keuze: solve-moves wél of niet opnemen in history
- Motivatie
- Gedrag na solve: kan de gebruiker undo gebruiken?

---

## 6. Aanbevolen eindontwerp

- Definitieve architectuurkeuze
- Welke bestaande onderdelen worden hergebruikt (concreet benoemen)
- Welke nieuwe onderdelen worden toegevoegd (minimaal houden)
- Belangrijkste trade-offs
- Waarom dit de eenvoudigste correcte keuze is voor `Cubie`

---

# Kwaliteitseisen

- Eén gestructureerd Markdown document
- Technisch diepgaand
- Alle aannames expliciet
- Trade-offs benoemd
- Schrijf in het Nederlands
- Wees beslissend, niet alleen beschrijvend
- Complexiteit minimaliseren is een expliciete kwaliteitseis

---

# Extra instructie (belangrijk)

Beoordeel alle keuzes alsof:
- de codebase langdurig onderhouden wordt
- meerdere ontwikkelaars samenwerken
- testbaarheid en voorspelbaarheid belangrijker zijn dan snelle implementatie
- **eenvoud zwaarder weegt dan volledigheid**
