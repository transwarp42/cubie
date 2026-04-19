# Ontwerpdocument: Undo/Redo Architectuur voor Cubie

## 0. Renderingkeuze

### Gekozen aanpak: unlit rendering via StandardMaterial

Het project gebruikt reeds `StandardMaterial` met `unlit: true` voor alle sticker-materialen. Deze keuze blijft ongewijzigd voor de undo/redo-uitbreiding.

### Waarom unlit voldoet

Stickerkleuren zijn de primaire visuele feedback bij undo/redo. Een undo-actie moet visueel exact dezelfde toestand herstellen als eerder zichtbaar was. Met `unlit: true` is de weergegeven kleur uitsluitend bepaald door `base_color`, onafhankelijk van camerahoek, lichtpositie of schaduwcondities. Dit garandeert **deterministische visuele output**: dezelfde logische toestand levert altijd dezelfde pixelkleuren op, ongeacht wanneer of vanuit welke hoek de gebruiker kijkt.

### Waarom PBR ongeschikt is

PBR-rendering (de standaard lit modus van `StandardMaterial`) berekent kleur op basis van lichtrichting, normalen, roughness en metallic-waarden. Na een undo kan een sticker op een andere oriëntatie ten opzichte van het licht staan dan bij de originele actie, waardoor de visuele kleur verschilt — ook al is de logische staat identiek. Dit breekt de verwachting van de gebruiker dat undo de "vorige toestand" exact herstelt. Emissive-gebaseerde benaderingen lijden onder tone mapping en bloom-effecten. Een custom unlit shader is functioneel equivalent aan `unlit: true` maar voegt onnodige complexiteit toe.

### Relevantie voor undo/redo

Undo/redo heeft geen directe impact op materialen of shaders. De renderingkeuze is echter een randvoorwaarde: omdat acties deterministisch en rendering-onafhankelijk moeten zijn, is het essentieel dat de visuele laag geen variabelen introduceert die de perceptie van "dezelfde toestand" beïnvloeden. Unlit rendering borgt dit.

---

## 1. Stapsgewijs Ontwikkelplan

### Fase 1: Actie-definitie en inversielogica

**Doel**: Een uniform actie-type definiëren dat zowel face-rotaties als volledige kubusrotaties representeert, inclusief inversielogica.

**Werkzaamheden**:
- Breid het bestaande `CubeMove`-concept uit met een variant voor volledige kubusrotatie. Een volledige kubusrotatie is equivalent aan drie gelijktijdige laagrotaties op dezelfde as (lagen -1, 0, 1).
- Definieer de inverse van elke actie: voor een face-rotatie is de inverse dezelfde as en laag met omgekeerde richting. Voor een kubusrotatie geldt hetzelfde principe op alle drie de lagen.
- Zorg dat de actie-representatie puur data is, zonder referenties naar entities of rendering-state.

**Risico's**:
- Volledige kubusrotatie bestaat nog niet in de huidige code. De logica in `CubeState.apply_rotation` werkt per laag; drie opeenvolgende aanroepen moeten correct samenwerken.
- De huidige `DragPhase` state machine ondersteunt alleen face-rotatie. Uitbreiding voor kubusrotatie vereist een tweede invoermechanisme (bijv. rechtermuisknop of modifier-toets).

**Validatie**: Verifieer dat het uitvoeren van een actie gevolgd door de inverse altijd exact de oorspronkelijke `CubeState` oplevert. Dit kan getest worden als pure unit test op het datamodel, zonder ECS.

### Fase 2: Undo/redo-stacks als ECS Resource

**Doel**: Twee stacks (undo en redo) implementeren als een enkele Bevy Resource.

**Werkzaamheden**:
- Maak een nieuwe Resource ("ActionHistory") die twee `Vec`-stacks bevat: één voor uitgevoerde acties (undo-stack) en één voor ongedaan gemaakte acties (redo-stack).
- Implementeer de vier flows: (1) nieuwe actie → push undo, clear redo; (2) undo → pop undo, push redo; (3) redo → pop redo, push undo; (4) nieuwe actie na undo → clear redo.
- Integreer met het bestaande `finish_face_rotation`-systeem: na succesvolle afronding van een animatie wordt de actie op de undo-stack gepusht.

**Risico's**:
- Geheugengebruik bij zeer lange sessies. Mitigatie: optionele maximale stackdiepte (bijv. 200 acties). Voor een Rubik's Cube is dit ruim voldoende.
- Race condition als een nieuwe actie start terwijl de vorige nog finaliseert. Mitigatie: de bestaande blokkering via `DragPhase::Animating` voorkomt dit reeds.

**Validatie**: Voer een reeks van N acties uit, undo N keer, verifieer dat `CubeState` gelijk is aan de opgeloste staat. Redo N keer, verifieer dat de eindtoestand identiek is aan de toestand na de N acties.

### Fase 3: Undo/redo-uitvoering via animatiepipeline

**Doel**: Undo- en redo-acties door dezelfde animatiepipeline leiden als reguliere acties.

**Werkzaamheden**:
- Introduceer een trigger-mechanisme (veld op de history-resource) dat aangeeft dat een undo of redo is aangevraagd.
- Een nieuw systeem leest dit trigger-signaal, haalt de betreffende actie (of inverse) op, en voedt deze in het bestaande rotatie/animatie-pad: `start_face_rotation` → `animate_face_rotation` → `finish_face_rotation`.
- Bij undo: de inverse van de actie wordt geanimeerd. Bij redo: de originele actie wordt opnieuw geanimeerd.
- Het `finish_face_rotation`-systeem moet weten of de zojuist voltooide actie een reguliere, undo-, of redo-actie was, zodat de stacks correct worden bijgewerkt (een reguliere actie pusht naar undo en cleart redo; een undo-actie pusht naar redo; een redo-actie pusht naar undo).

**Risico's**:
- Het bestaande systeem in `start_face_rotation` verwacht een `DragPhase::Resolved`. Undo/redo moet een alternatief pad bieden dat niet via drag-interactie loopt. Oplossing: een tweede trigger-bron naast `DragState`, of een gedeeld "pending move"-concept.
- Animatieduur bij undo/redo: dezelfde duur als reguliere rotatie (0.3s) of sneller? Ontwerpkeuze.

**Validatie**: Voer een actie uit, druk undo, verifieer visueel en logisch dat de kubus terugkeert. Druk redo, verifieer dat de kubus weer vooruit gaat.

### Fase 4: UI-knoppen

**Doel**: Undo- en redo-knoppen toevoegen aan de bestaande UI.

**Werkzaamheden**:
- In `labels.rs` of een nieuw bestand: spawn twee knoppen. Redo-knop links van undo-knop, undo-knop links van de bestaande "Labels"-toggleknop (die nu rechtsbovenin staat op `right: 10px`).
- Knoppen zijn visueel inactief (gedimde kleur, geen interactie) wanneer de bijbehorende stack leeg is. Een systeem leest de lengte van de undo/redo-stacks en past `BackgroundColor` en `Interaction`-filtering aan.
- Bij klik: stuur het trigger-signaal uit fase 3.

**Risico's**:
- Positionering: de huidige Labels-knop gebruikt absolute positionering (`top: 10px, right: 10px`). De undo/redo-knoppen moeten links ervan komen, met vaste pixel-offsets of via een flex-container.
- Klik op een knop mag niet doorpropageren naar de kubus-picking.

**Validatie**: Verifieer dat knoppen correct activeren/deactiveren, dat klikken op inactieve knoppen niets doen, en dat de visuele staat klopt.

### Fase 5: Volledige kubusrotatie als actie

**Doel**: Volledige kubusrotatie (niet slechts één laag) toevoegen als undo/redo-actie.

**Werkzaamheden**:
- Definieer kubusrotatie als actie-variant: alle 27 cubies roteren rond een as.
- Animatie: dezelfde pivot-aanpak maar met alle cubies als kinderen.
- Invoer: rechtermuisknop-drag of andere differentiator ten opzichte van face-rotatie.
- De actie wordt op dezelfde undo-stack gepusht als face-rotaties.

**Risico's**:
- Kubusrotatie verandert `grid_position` van alle 27 cubies en alle face-directions. De inverse moet dit exact terugdraaien.
- Ontwerpkeuze: tellen kubusrotaties als undo-acties? Aanbeveling: wél undo-baar, voor consistentie.

**Validatie**: Kubusrotatie uitvoeren, undo, verifieer dat alle 27 cubies exact terugkeren.

---

## 2. Architectuurschets

### Modulestructuur (uitgebreid)

```
src/
├── main.rs                 # App-configuratie, resource-registratie
├── camera.rs               # Orbit-camera (ongewijzigd)
├── icon.rs                 # App-icoon (ongewijzigd)
└── cube/
    ├── mod.rs              # Module-declaraties (uitgebreid met history)
    ├── model.rs            # Datamodel: CubeState, CubeMove, actie-definitie (uitgebreid)
    ├── spawn.rs            # Spawning (ongewijzigd)
    ├── picking.rs          # Raycasting (ongewijzigd)
    ├── input.rs            # Drag-detectie (aangepast: undo/redo trigger)
    ├── rotation.rs         # Rotatielogica (aangepast: alternatief invoerpad)
    ├── animation.rs        # Animatie-interpolatie (minimaal aangepast)
    ├── labels.rs           # Face labels (ongewijzigd)
    └── history.rs          # NIEUW: undo/redo stacks, UI-knoppen, trigger-systeem
```

### Nieuwe en aangepaste Resources

**ActionHistory (Resource) — NIEUW**
- Undo-stack: geordende lijst van uitgevoerde acties (meest recente bovenaan)
- Redo-stack: geordende lijst van ongedaan gemaakte acties
- Actieve-bron: vlag die aangeeft of de huidige animatie een reguliere actie, undo, of redo betreft

**FaceRotationAnimation (Resource) — AANGEPAST**
- Uitgebreid met een veld dat de oorsprong van de actie aangeeft (regulier, undo, redo), zodat `finish_face_rotation` weet naar welke stack de actie moet

**DragState (Resource) — MINIMAAL AANGEPAST**
- Geen structurele wijziging; het bestaande `DragPhase::Resolved` pad blijft voor muisinvoer
- Undo/redo omzeilt `DragState` en voedt de actie direct in de animatie-resource

### Nieuwe Components

**UndoButton / RedoButton (Component) — NIEUW**
- Marker-components voor de UI-knoppen, analoog aan `ToggleLabelsButton`

### Systeemoverzicht en executievolgorde

```
Systems (Update, geketend):

1. handle_mouse_input          — leest muisinvoer, beheert DragState
2. resolve_drag_direction      — bepaalt rotatie-as bij drag
3. handle_undo_redo_input      — NIEUW: leest knopinteractie of toetsenbordinvoer,
                                  haalt actie van stack, schrijft naar animatie-resource
4. start_face_rotation         — AANGEPAST: leest DragState::Resolved OF undo/redo-trigger
5. animate_face_rotation       — interpoleert pivot-rotatie (ongewijzigd)
6. finish_face_rotation        — AANGEPAST: pusht actie naar correcte stack
7. orbit_camera_system         — camera-orbit
8. update_face_labels          — label-projectie
9. toggle_labels_button        — label-toggle
10. update_undo_redo_buttons   — NIEUW: activeer/deactiveer knoppen op basis van stackgrootte
```

### Dataflow

```
Muisinvoer (drag op kubus)
    │
    ▼
handle_mouse_input → DragState::Pending
    │
    ▼
resolve_drag_direction → DragState::Resolved { actie }
    │
    ▼                                    UI-knop of toetsenbord (Ctrl+Z / Ctrl+Y)
start_face_rotation ◄────────────────── handle_undo_redo_input
    │                                        │ leest ActionHistory
    │ maakt pivot, reparent cubies            │ bepaalt inverse (undo) of origineel (redo)
    │ initialiseert FaceRotationAnimation     │ schrijft naar FaceRotationAnimation
    ▼
animate_face_rotation (elke frame)
    │ interpoleert pivot via Quat::slerp
    ▼
finish_face_rotation
    │ deparent cubies, update CubeState
    │ pusht actie naar correcte stack:
    │   regulier → undo-stack (clear redo)
    │   undo    → redo-stack
    │   redo    → undo-stack
    ▼
update_undo_redo_buttons
    │ leest stackgrootte → activeert/deactiveert knoppen
    ▼
Rendering (automatisch door Bevy)
```

---

## 3. Undo/Redo Model

### Representatie van acties

Een actie is een puur data-object met de volgende informatie:
- **Rotatie-as**: X, Y of Z (overeenkomend met het bestaande `RotationAxis`)
- **Laag**: het gehele getal (-1, 0, of 1) dat aangeeft welke slice roteert. Voor een volledige kubusrotatie: een speciale markering die aangeeft dat alle drie de lagen (−1, 0, 1) roteren.
- **Richting**: kloksgewijs of tegen de klok in (boolean)

Dit is in essentie het bestaande `CubeMove`, uitgebreid met een variant voor volledige kubusrotatie. De representatie bevat geen Entity-referenties, geen timestamps, geen rendering-data — uitsluitend de minimale informatie die nodig is om de actie deterministisch te reproduceren en te inverteren.

### Inversielogica

De inverse van een actie is triviaal:
- **Face-rotatie**: zelfde as, zelfde laag, omgekeerde richting (kloksgewijs ↔ tegen de klok in)
- **Kubusrotatie**: zelfde as, omgekeerde richting op alle lagen

Dit maakt het systeem eenvoudig en foutbestendig. Er is geen noodzaak om de volledige kubustoestand als snapshot op te slaan: de inverse-operatie is goedkoop te berekenen en altijd correct dankzij de deterministische aard van `CubeState.apply_rotation`.

### Opslag in stacks

Twee `Vec`-structuren binnen een enkele Resource:
- **Undo-stack**: groeit bij elke nieuwe actie (push naar het einde), undo popt van het einde
- **Redo-stack**: gevuld bij undo, geleegd bij een nieuwe reguliere actie

Geheugenimpact is verwaarloosbaar: elke actie is slechts drie velden (as, laag, richting). Zelfs duizenden acties gebruiken minder dan een kilobyte.

### Waarom command-based en niet snapshot-based

| Criterium | Command-based | Snapshot-based | Hybrid |
|-----------|--------------|----------------|--------|
| Geheugen | Minimaal (enkele bytes per actie) | Hoog (27 cubies × sticker-data per snapshot) | Gemiddeld |
| Determinisme | Gegarandeerd (acties zijn per definitie deterministisch) | Niet nodig (staat wordt direct hersteld) | Afhankelijk van implementatie |
| Complexiteit | Laag (inverse is triviaal) | Laag (kopieer staat) | Hoog (twee mechanismen) |
| ECS-compatibiliteit | Excellent (acties zijn pure data, geen entity-referenties) | Matig (snapshot moet entities opnieuw synchroniseren) | Gemiddeld |
| Performance | O(1) per undo/redo (één rotatie toepassen) | O(n) per undo/redo (volledige staat kopiëren, alle entities synchroniseren) | Variabel |

**Keuze: command-based.** De acties in Cubie zijn inherent inverteerbaar (rotaties zijn hun eigen inverse met omgekeerde richting), klein in dataomvang, en deterministisch. Snapshot-based zou onnodige complexiteit en geheugenoverhead toevoegen zonder voordelen. Een hybrid-aanpak is niet gerechtvaardigd bij zo'n eenvoudig actiemodel.

### Integratie met ECS

**Actieopslag**: Resource (niet Events). Events in Bevy zijn éénmalig en worden na twee frames opgeruimd. Stacks moeten persistent zijn over de hele sessie; een Resource is hiervoor de juiste keuze.

**Trigger-mechanisme**: Een veld op de ActionHistory-resource (bijv. "pending_undo: bool"). Eenvoudig, direct leesbaar door het rotatie-systeem. Een Event-gebaseerde aanpak is meer idiomatisch ECS maar voegt een extra synchronisatiestap toe die hier niet nodig is.

**Stateless systems**: Alle systemen lezen/schrijven resources en components zonder interne mutable state. De ActionHistory-resource is de enige bron van waarheid voor de actiegeschiedenis. Systemen beslissen op basis van de huidige waarden in de resource, niet op basis van eerder opgeslagen lokale variabelen.

**Vermijden van tight coupling**: Het undo/redo-systeem kent alleen het actie-datatype en de ActionHistory-resource. Het weet niet hoe animatie werkt, welke entities betrokken zijn, of hoe picking functioneert. De koppeling loopt uitsluitend via de gedeelde actie-representatie en de animatie-resource — dezelfde interface die de reguliere invoer al gebruikt.

### Edge cases

**Nieuwe actie tijdens undo-state**: De redo-stack wordt gewist. Dit is standaardgedrag en voorkomt een vertakkende actiegeschiedenis. De gebruiker verliest de mogelijkheid om eerder ongedane acties te redo-en, wat de verwachte UX is.

**Undo tijdens lopende animatie**: De huidige architectuur blokkeert alle invoer tijdens animatie (`DragPhase::Animating` voorkomt nieuwe drag-interacties). Dezelfde blokkering geldt voor undo/redo-verzoeken: het `handle_undo_redo_input`-systeem checkt of `FaceRotationAnimation.active` waar is en negeert verzoeken in dat geval. Alternatief (geavanceerder): de lopende animatie direct voltooien (skip naar eindtoestand) en dan de undo starten. Dit is een betere UX maar complexer. **Aanbeveling**: start met blokkering, overweeg skip-to-end als verbetering.

**Snelle invoer / race conditions**: Door de systeemketening (`chain()`) is de executievolgorde deterministisch binnen één frame. Er bestaan geen race conditions: systemen draaien sequentieel. Een undo-verzoek en een drag-verzoek in hetzelfde frame worden verwerkt in volgorde; de eerste die de animatie start wint, de tweede wordt genegeerd door de animatie-actief-check.

**Determinisme**: Gegarandeerd door de aard van het actiemodel. `CubeState.apply_rotation` is een pure functie op de kubustoestand. Dezelfde reeks acties levert altijd dezelfde eindtoestand, ongeacht timing, framerate of invoermethode.

---

## 4. Libraries en Tooling

### Bevy (v0.15) — reeds in gebruik

**Voordelen**: Volledige ECS-infrastructuur, UI-systeem voor knoppen, resource-management, systeem-scheduling met `chain()`. Alles wat nodig is voor undo/redo is beschikbaar zonder externe dependencies.

**Nadelen**: Geen ingebouwd undo/redo-framework; moet zelf geïmplementeerd worden. Dit is echter triviaal gezien de eenvoud van het actiemodel.

### bevy_mod_picking — niet nodig voor undo/redo

De huidige code gebruikt custom raycasting. Undo/redo heeft geen picking-requirements; de bestaande aanpak blijft ongewijzigd.

### bevy_egui — optioneel voor debug UI

**Voordelen**: Immediate-mode UI, handig om de undo/redo-stacks visueel te inspecteren tijdens ontwikkeling.

**Nadelen**: Extra dependency, overkill voor twee knoppen. De knoppen zelf worden beter met Bevy's native UI gebouwd, consistent met de bestaande Labels-toggleknop.

**Aanbeveling**: Niet gebruiken voor de productie-UI. Eventueel tijdelijk toevoegen voor debugging.

### Externe undo/redo crates (bijv. `undo`, `redo`)

**Voordelen**: Generieke undo/redo-functionaliteit, mogelijk met merge/groepering van acties.

**Nadelen**: Niet Bevy-aware; integratie met ECS-resources vereist wrapper-code. De undo/redo-logica voor Cubie is zo eenvoudig (twee vecs, push/pop) dat een externe crate meer complexiteit toevoegt dan het oplost.

**Aanbeveling**: Niet gebruiken. De implementatie is triviaal genoeg om zelf te doen, met volledige controle over ECS-integratie.

### Toetsenbordsneltoetsen

Bevy's ingebouwde `ButtonInput<KeyCode>` volstaat voor Ctrl+Z (undo) en Ctrl+Y of Ctrl+Shift+Z (redo). Geen externe input-library nodig.

---

## Aannames

1. Er is altijd maximaal één animatie tegelijk actief (bestaande architectuur-aanname, ongewijzigd).
2. Volledige kubusrotatie wordt als undo-bare actie beschouwd, niet als view-only operatie.
3. Er is geen maximale stackdiepte vereist; een optionele limiet kan later worden toegevoegd.
4. De bestaande systeemketening via `chain()` blijft behouden; alle systemen draaien sequentieel.
5. Toetsenbordsneltoetsen (Ctrl+Z, Ctrl+Y) worden naast UI-knoppen ondersteund.

## Trade-offs

| Keuze | Voordeel | Nadeel |
|-------|----------|--------|
| Command-based i.p.v. snapshot | Minimaal geheugen, triviale inverse | Vereist dat alle acties inverteerbaar zijn (geldt voor rotaties) |
| Blokkering tijdens animatie | Eenvoudige state machine, geen conflicten | Gebruiker moet wachten; snelle undo-undo voelt traag |
| Resource i.p.v. Events voor stacks | Persistent, direct leesbaar | Minder "ECS-idiomatisch" dan events; geen fan-out |
| Undo/redo via bestaande animatiepipeline | Hergebruik, consistente visuele feedback | Undo duurt 0.3s; instant undo zou sneller aanvoelen |
| Kubusrotatie als undo-bare actie | Volledige consistentie | Lange undo-keten als gebruiker veel roteert |
| Geen externe undo-crate | Geen dependency, volledige controle | Zelf implementeren (maar triviaal) |

