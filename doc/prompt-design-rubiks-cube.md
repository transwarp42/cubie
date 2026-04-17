Je bent een ervaren Rust graphics programmeur en game engine architect.

Doel:
Ontwikkel een Rust applicatie die een 3x3 Rubik’s Cube in 3D op het scherm rendert en deze als geheel met de muis laat roteren.

Randvoorwaarden:
- Programmeertaal: Rust
- De applicatie toont een 3x3 Rubik’s Cube in 3D
- De applicatie draait niet in een browser
- De gebruiker kan met de muis de volledige kubus vrij draaien en bekijken (orbit camera)
- In deze versie kunnen individuele zijden van de kubus NIET draaien
- De architectuur moet zodanig opgezet zijn dat het later eenvoudig is om:
  - individuele zijden (faces/layers) te roteren
  - animaties toe te voegen
  - cube-states te beheren

Technische richtlijnen:
- Gebruik een moderne Rust 3D stack (bijv. wgpu + winit of een engine zoals Bevy)
- Scheid rendering, input, en cube-logica duidelijk
- Modelleer de Rubik’s Cube als losse cubies (3x3x3)
- Maak abstraheringen voor:
  - CubeState
  - Cubie
  - Transform / rotation logic
- Muisslepen roteert alleen de camera (niet de cube-definitie)

Wat ik van je verwacht:
- Een stapsgewijs ontwikkelplan
- Architectuurschets (modules / structs)
- Motivatie voor gekozen libraries
- Aandacht voor uitbreidbaarheid naar zijde-rotaties
- Zet het resultaat in een mark-down bestand

Antwoord duidelijk, technisch onderbouwd en in het Nederlands.