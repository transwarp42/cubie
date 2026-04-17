# Plan: Rubik's Cube 3D Renderer in Rust (Bevy)

Bevy 0.18.1 applicatie met `bevy_panorbit_camera` voor orbit-camera. 27 cubies gerenderd als donkergrijs lichaam + gekleurde sticker-children. Architectuur klaar voor face-rotaties en animaties.

## Fase 1 — Project Setup
1. **Cargo.toml**: voeg `bevy = "0.18"` en `bevy_panorbit_camera = "0.34"` toe
2. Maak modulaire mappen: `src/cube/` en `src/camera/`

## Fase 2 — Data Model (`src/cube/cubie.rs`)
3. `FaceColor` enum: `White, Yellow, Green, Blue, Red, Orange, Black`
4. `Face` enum: `Top, Bottom, Front, Back, Right, Left` (elk met een offset-vector)
5. `Cubie` struct: `logical_pos: (i8,i8,i8)` + `face_colors: [FaceColor; 6]`
6. `CubeState` struct: `cubies: Vec<Cubie>` + `fn solved() -> CubeState`
7. ECS-markers: `CubieMarker { pos: (i8,i8,i8) }`, `StickerMarker { face: Face, cubie_pos: (i8,i8,i8) }`

## Fase 3 — Rendering (`src/cube/mod.rs`, `src/camera/mod.rs`)
8. **`spawn_cube` systeem**: voor elk cubie spawn:
   - donkergrijs `Cuboid(0.95)` als parent entity
   - per gekleurde face een dun `Cuboid(0.9, 0.9, 0.02)` als child (geroteerd + offset per face)
9. **`spawn_camera` systeem**: `PanOrbitCamera` component + `DirectionalLight`

## Fase 4 — Uitbreidbaarheid-stubs (`src/cube/moves.rs`) *(parallel met fase 3)*
10. `Move` enum: `FaceRotation { face: Face, clockwise: bool }`
11. `fn apply_move(state: &mut CubeState, m: Move)` stub
12. `AnimationProgress` resource placeholder: `{ active: Option<Move>, t: f32 }`

## Fase 5 — Plugin-integratie (`src/main.rs`)
13. `CubePlugin` + `CameraPlugin` als Bevy plugins
14. `App::new().add_plugins((DefaultPlugins, PanOrbitCameraPlugin, CubePlugin, CameraPlugin)).run()`

## Relevante bestanden
- `Cargo.toml` — dependencies
- `src/main.rs` — App + plugins
- `src/cube/mod.rs` — CubePlugin, spawn_cube
- `src/cube/cubie.rs` — Cubie, FaceColor, Face, CubeState
- `src/cube/moves.rs` — Move, apply_move stub, AnimationProgress
- `src/camera/mod.rs` — CameraPlugin, spawn_camera

## Verificatie
1. `cargo build` slaagt zonder errors
2. Venster toont 3x3 Rubik's Cube met 6 correcte kleuren
3. Linkermuisknop slepen roteert de camera
4. Scroll zoomt in/uit

## Kleurschema
Top=Wit, Bottom=Geel, Front=Groen, Back=Blauw, Right=Rood, Left=Oranje

## Buiten scope (v1)
Face-rotaties uitvoeren, animaties afspelen, cube solver, touch input