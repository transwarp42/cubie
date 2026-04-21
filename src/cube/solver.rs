use std::collections::VecDeque;

use bevy::prelude::*;
use rcuber::cubie::CubieCube;
use rcuber::facelet::FaceCube;
use rcuber::solver::min2phase::Min2PhaseSolver;

use crate::camera::OrbitCamera;
use super::animation::{ActionOrigin, FaceRotationAnimation};
use super::model::*;
use super::rotation::RotationPivot;
use super::scramble::{rcuber_move_to_cube_moves, ScrambleQueue, ScrambleStatus};

/// Status of the solve state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SolveStatus {
    #[default]
    Idle,
    Scanning,  // Visueel "fotograferen" van elke zijde
    Active,    // Oplossing uitvoeren
}

/// Resource tracking the solve queue and status — direct analogue of ScrambleQueue.
#[derive(Resource, Default)]
pub struct SolveQueue {
    pub status: SolveStatus,
    pub moves: VecDeque<CubeMove>,
}

/// Resource tracking the scan animation state.
#[derive(Resource)]
pub struct ScanAnimation {
    pub active: bool,
    pub current_face: usize,
    pub elapsed: f32,
    pub duration_per_face: f32,
    pub flash_triggered: bool,   // Of flash al is getriggerd voor huidige face
    pub flash_active: bool,       // Of flash momenteel actief is
    pub flash_elapsed: f32,       // Verstreken tijd voor flash animatie
    pub returning_to_start: bool, // Of we terug aan het draaien zijn naar start
    pub saved_orbit_state: Option<(f32, f32, f32)>, // (yaw, pitch, distance)
    pub from_yaw: f32,            // Camera yaw aan begin van huidige transitie
    pub from_pitch: f32,          // Camera pitch aan begin van huidige transitie
}

impl Default for ScanAnimation {
    fn default() -> Self {
        Self {
            active: false,
            current_face: 0,
            elapsed: 0.0,
            duration_per_face: 0.5, // 0.5s per zijde
            flash_triggered: false,
            flash_active: false,
            flash_elapsed: 0.0,
            returning_to_start: false,
            saved_orbit_state: None,
            from_yaw: 0.0,
            from_pitch: 0.0,
        }
    }
}

/// Camera yaw/pitch targets for each face (perpendicular view, no other faces visible).
/// Front(Z+), Right(X+), Back(Z-), Left(X-), Up(Y+), Down(Y-)
const FACE_YAW: [f32; 6] = [
    0.0,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::PI,
    -std::f32::consts::FRAC_PI_2,
    0.0,
    0.0,
];
const FACE_PITCH: [f32; 6] = [
    0.0,
    0.0,
    0.0,
    0.0,
    std::f32::consts::FRAC_PI_2 - 0.01,
    -(std::f32::consts::FRAC_PI_2 - 0.01),
];

/// Marker component for the camera flash overlay.
#[derive(Component)]
pub struct FlashOverlay;

impl SolveQueue {
    pub fn is_active(&self) -> bool {
        self.status == SolveStatus::Active
    }
}

/// Marker for the solve button.
#[derive(Component)]
pub struct SolveButton;

/// Spawn the Solve button (left of Scramble).
pub fn spawn_solve_button(mut commands: Commands) {
    commands
        .spawn((
            SolveButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(246.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(Color::srgba(0.2, 0.25, 0.2, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Solve"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ));
        });

    // Spawn flash overlay (invisible by default)
    commands.spawn((
        FlashOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)), // Transparent white
        ZIndex(1000), // Above everything
        Visibility::Hidden,
    ));
}

/// System: handle Solve button click — start scanning animation.
pub fn handle_solve_input(
    mut solve: ResMut<SolveQueue>,
    scramble: Res<ScrambleQueue>,
    animation: Res<FaceRotationAnimation>,
    query: Query<&Interaction, (Changed<Interaction>, With<SolveButton>)>,
) {
    if solve.status != SolveStatus::Idle
        || scramble.status != ScrambleStatus::Idle
        || animation.active
    {
        return;
    }

    for interaction in &query {
        if *interaction == Interaction::Pressed {
            // Start scanning phase
            solve.status = SolveStatus::Scanning;
        }
    }
}

/// Build solve moves by reading the VISUAL cube state from entity transforms.
///
/// For each cubie, `Transform.rotation` encodes the cumulative rotation.
/// The current facing direction of a sticker = snap(rotation * initial_normal).
fn compute_solve_moves(
    cubies: &Query<(&Cubie, &Transform, &Children)>,
    stickers: &Query<&Sticker>,
) -> VecDeque<CubeMove> {
    let face_string = cubies_to_face_string(cubies, stickers);
    let Ok(face_cube) = FaceCube::try_from(face_string.as_str()) else {
        warn!("Solver: invalid facelet string: {}", face_string);
        return VecDeque::new();
    };
    let Ok(cubie_cube) = CubieCube::try_from(&face_cube) else {
        warn!("Solver: could not build CubieCube from face string: {}", face_string);
        return VecDeque::new();
    };

    if cubie_cube == CubieCube::default() {
        return VecDeque::new();
    }

    let mut solver = Min2PhaseSolver { cube: cubie_cube };
    let solution = solver.solve();

    solution
        .moves
        .iter()
        .flat_map(|&m| rcuber_move_to_cube_moves(m))
        .collect()
}

/// Build the 54-char rcuber facelet string from visual entity state.
///
/// The rcuber convention (fixed center orientation):
///   U = White, R = Red, F = Green, D = Yellow, L = Orange, B = Blue
///
/// For each cubie, its stickers' initial normals are rotated by the cubie's
/// accumulated `Transform.rotation` to obtain their current facing direction.
///
/// This function handles middle layer rotations by detecting the actual center
/// positions and mapping colors accordingly.
fn cubies_to_face_string(
    cubies: &Query<(&Cubie, &Transform, &Children)>,
    stickers: &Query<&Sticker>,
) -> String {
    // Build lookup: grid_position → [(current_face_direction, color)]
    let mut lookup: std::collections::HashMap<IVec3, Vec<(FaceDirection, StickerColor)>> =
        Default::default();

    for (cubie, transform, children) in cubies.iter() {
        let mut list = Vec::new();
        for &child in children.iter() {
            if let Ok(sticker) = stickers.get(child) {
                let rotated = transform.rotation * sticker.face_direction.normal();
                let current_dir = FaceDirection::from_normal(rotated);
                list.push((current_dir, sticker.color));
            }
        }
        lookup.insert(cubie.grid_position, list);
    }

    // Detect which colors are currently at the center positions to handle
    // middle layer rotations. The rcuber solver expects fixed centers:
    // U=White, R=Red, F=Green, D=Yellow, L=Orange, B=Blue
    let center_map = detect_center_mapping(&lookup);

    #[rustfmt::skip]
    let face_scan: &[(IVec3, FaceDirection)] = &[
        // U face — looking from above, back row first (z=-1), left→right (x=-1..1)
        (IVec3::new(-1, 1,-1), FaceDirection::Up),  (IVec3::new(0, 1,-1), FaceDirection::Up),  (IVec3::new(1, 1,-1), FaceDirection::Up),
        (IVec3::new(-1, 1, 0), FaceDirection::Up),  (IVec3::new(0, 1, 0), FaceDirection::Up),  (IVec3::new(1, 1, 0), FaceDirection::Up),
        (IVec3::new(-1, 1, 1), FaceDirection::Up),  (IVec3::new(0, 1, 1), FaceDirection::Up),  (IVec3::new(1, 1, 1), FaceDirection::Up),
        // R face — looking from right, top→bottom (y=1..-1), front→back (z=1..-1)
        (IVec3::new(1, 1, 1),  FaceDirection::Right), (IVec3::new(1, 1, 0),  FaceDirection::Right), (IVec3::new(1, 1,-1), FaceDirection::Right),
        (IVec3::new(1, 0, 1),  FaceDirection::Right), (IVec3::new(1, 0, 0),  FaceDirection::Right), (IVec3::new(1, 0,-1), FaceDirection::Right),
        (IVec3::new(1,-1, 1),  FaceDirection::Right), (IVec3::new(1,-1, 0),  FaceDirection::Right), (IVec3::new(1,-1,-1), FaceDirection::Right),
        // F face — looking from front, top→bottom (y=1..-1), left→right (x=-1..1)
        (IVec3::new(-1, 1, 1), FaceDirection::Front), (IVec3::new(0, 1, 1), FaceDirection::Front), (IVec3::new(1, 1, 1), FaceDirection::Front),
        (IVec3::new(-1, 0, 1), FaceDirection::Front), (IVec3::new(0, 0, 1), FaceDirection::Front), (IVec3::new(1, 0, 1), FaceDirection::Front),
        (IVec3::new(-1,-1, 1), FaceDirection::Front), (IVec3::new(0,-1, 1), FaceDirection::Front), (IVec3::new(1,-1, 1), FaceDirection::Front),
        // D face — front row first (z=1), left→right
        (IVec3::new(-1,-1, 1), FaceDirection::Down),  (IVec3::new(0,-1, 1), FaceDirection::Down),  (IVec3::new(1,-1, 1), FaceDirection::Down),
        (IVec3::new(-1,-1, 0), FaceDirection::Down),  (IVec3::new(0,-1, 0), FaceDirection::Down),  (IVec3::new(1,-1, 0), FaceDirection::Down),
        (IVec3::new(-1,-1,-1), FaceDirection::Down),  (IVec3::new(0,-1,-1), FaceDirection::Down),  (IVec3::new(1,-1,-1), FaceDirection::Down),
        // L face — looking from left, top→bottom (y=1..-1), back→front (z=-1..1)
        (IVec3::new(-1, 1,-1), FaceDirection::Left),  (IVec3::new(-1, 1, 0), FaceDirection::Left),  (IVec3::new(-1, 1, 1), FaceDirection::Left),
        (IVec3::new(-1, 0,-1), FaceDirection::Left),  (IVec3::new(-1, 0, 0), FaceDirection::Left),  (IVec3::new(-1, 0, 1), FaceDirection::Left),
        (IVec3::new(-1,-1,-1), FaceDirection::Left),  (IVec3::new(-1,-1, 0), FaceDirection::Left),  (IVec3::new(-1,-1, 1), FaceDirection::Left),
        // B face — looking from back, top→bottom (y=1..-1), right→left (x=1..-1, mirrored)
        (IVec3::new(1, 1,-1),  FaceDirection::Back),  (IVec3::new(0, 1,-1),  FaceDirection::Back),  (IVec3::new(-1, 1,-1), FaceDirection::Back),
        (IVec3::new(1, 0,-1),  FaceDirection::Back),  (IVec3::new(0, 0,-1),  FaceDirection::Back),  (IVec3::new(-1, 0,-1), FaceDirection::Back),
        (IVec3::new(1,-1,-1),  FaceDirection::Back),  (IVec3::new(0,-1,-1),  FaceDirection::Back),  (IVec3::new(-1,-1,-1), FaceDirection::Back),
    ];

    let mut s = String::with_capacity(54);
    for &(pos, face_dir) in face_scan {
        let ch = lookup
            .get(&pos)
            .and_then(|list| list.iter().find(|(fd, _)| *fd == face_dir))
            .map(|&(_, sc)| sticker_color_to_rcuber_char_mapped(sc, &center_map))
            .unwrap_or('U');
        s.push(ch);
    }
    s
}

/// Detect which colors are currently at the center positions.
/// Returns a mapping from actual color to the face character it should represent.
fn detect_center_mapping(
    lookup: &std::collections::HashMap<IVec3, Vec<(FaceDirection, StickerColor)>>,
) -> std::collections::HashMap<StickerColor, char> {
    use std::collections::HashMap;

    // Center positions for each face direction
    let centers = [
        (IVec3::new(0, 1, 0), FaceDirection::Up, 'U'),      // White in solved state
        (IVec3::new(0, -1, 0), FaceDirection::Down, 'D'),   // Yellow in solved state
        (IVec3::new(1, 0, 0), FaceDirection::Right, 'R'),   // Red in solved state
        (IVec3::new(-1, 0, 0), FaceDirection::Left, 'L'),   // Orange in solved state
        (IVec3::new(0, 0, 1), FaceDirection::Front, 'F'),   // Green in solved state
        (IVec3::new(0, 0, -1), FaceDirection::Back, 'B'),   // Blue in solved state
    ];

    let mut map = HashMap::new();

    for &(pos, face_dir, face_char) in &centers {
        if let Some(stickers) = lookup.get(&pos) {
            if let Some(&(_, color)) = stickers.iter().find(|(fd, _)| *fd == face_dir) {
                map.insert(color, face_char);
            }
        }
    }

    map
}

/// Map our StickerColor to the rcuber facelet character, using the detected center mapping.
fn sticker_color_to_rcuber_char_mapped(
    color: StickerColor,
    center_map: &std::collections::HashMap<StickerColor, char>
) -> char {
    center_map.get(&color).copied().unwrap_or_else(|| {
        // Fallback to default mapping if not found
        match color {
            StickerColor::White  => 'U',
            StickerColor::Red    => 'R',
            StickerColor::Green  => 'F',
            StickerColor::Yellow => 'D',
            StickerColor::Orange => 'L',
            StickerColor::Blue   => 'B',
        }
    })
}

/// System: start scanning animation when solve is initiated.
pub fn start_scan_animation(
    mut solve: ResMut<SolveQueue>,
    mut scan: ResMut<ScanAnimation>,
    camera_query: Query<&OrbitCamera>,
) {
    if solve.status != SolveStatus::Scanning || scan.active {
        return;
    }

    // Save current camera orbit state so we can restore it afterwards
    if let Ok(orbit) = camera_query.get_single() {
        scan.saved_orbit_state = Some((orbit.yaw, orbit.pitch, orbit.distance));
        scan.from_yaw = orbit.yaw;
        scan.from_pitch = orbit.pitch;
    }

    scan.active = true;
    scan.current_face = 0;
    scan.elapsed = 0.0;
    scan.flash_triggered = false;
    scan.flash_active = false;
    scan.flash_elapsed = 0.0;
    scan.returning_to_start = false;
}

/// System: animate scanning through each face by moving the camera.
pub fn animate_scan(
    time: Res<Time>,
    mut scan: ResMut<ScanAnimation>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    if !scan.active {
        return;
    }

    let Ok((mut orbit, mut cam_tf)) = camera_query.get_single_mut() else {
        return;
    };

    scan.elapsed += time.delta_secs();
    let t = (scan.elapsed / scan.duration_per_face).min(1.0);
    // Ease-in-out cubic
    let t_eased = if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    };

    // Check if we're returning to start position
    if scan.returning_to_start {
        if let Some((saved_yaw, saved_pitch, saved_dist)) = scan.saved_orbit_state {
            orbit.yaw = scan.from_yaw + (saved_yaw - scan.from_yaw) * t_eased;
            orbit.pitch = scan.from_pitch + (saved_pitch - scan.from_pitch) * t_eased;
            orbit.distance = saved_dist;
        }
        let pos = Vec3::new(
            orbit.distance * orbit.pitch.cos() * orbit.yaw.sin(),
            orbit.distance * orbit.pitch.sin(),
            orbit.distance * orbit.pitch.cos() * orbit.yaw.cos(),
        );
        cam_tf.translation = pos;
        cam_tf.look_at(Vec3::ZERO, Vec3::Y);
        return;
    }

    if scan.current_face >= FACE_YAW.len() {
        if !scan.returning_to_start {
            scan.returning_to_start = true;
            scan.elapsed = 0.0;
            // Save current (last face) yaw/pitch as starting point for return
            scan.from_yaw = FACE_YAW[FACE_YAW.len() - 1];
            scan.from_pitch = FACE_PITCH[FACE_PITCH.len() - 1];
        }
        return;
    }

    let target_yaw = FACE_YAW[scan.current_face];
    let target_pitch = FACE_PITCH[scan.current_face];

    orbit.yaw = scan.from_yaw + (target_yaw - scan.from_yaw) * t_eased;
    orbit.pitch = scan.from_pitch + (target_pitch - scan.from_pitch) * t_eased;
    // Distance stays constant (no zoom)

    let pos = Vec3::new(
        orbit.distance * orbit.pitch.cos() * orbit.yaw.sin(),
        orbit.distance * orbit.pitch.sin(),
        orbit.distance * orbit.pitch.cos() * orbit.yaw.cos(),
    );
    cam_tf.translation = pos;
    cam_tf.look_at(Vec3::ZERO, Vec3::Y);

    // Trigger flash when face is fully visible (90% through animation)
    if t >= 0.9 && !scan.flash_triggered {
        scan.flash_triggered = true;
        scan.flash_active = true;
        scan.flash_elapsed = 0.0;
    }

    // Move to next face when current animation completes
    if scan.elapsed >= scan.duration_per_face {
        scan.elapsed = 0.0;
        scan.from_yaw = target_yaw;
        scan.from_pitch = target_pitch;
        scan.current_face += 1;
        scan.flash_triggered = false; // Reset voor volgende face
    }
}

/// System: animate the camera flash effect.
pub fn animate_camera_flash(
    time: Res<Time>,
    mut scan: ResMut<ScanAnimation>,
    mut flash_query: Query<(&mut BackgroundColor, &mut Visibility), With<FlashOverlay>>,
) {
    if !scan.flash_active {
        // Ensure flash is hidden when not active
        for (mut bg, mut vis) in &mut flash_query {
            *vis = Visibility::Hidden;
            bg.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
        }
        return;
    }

    scan.flash_elapsed += time.delta_secs();

    // Flash parameters
    let flash_duration = 0.15; // Total flash duration (150ms)
    let fade_in_duration = 0.05; // Quick fade in (50ms)
    let fade_out_duration = 0.10; // Slower fade out (100ms)

    let alpha = if scan.flash_elapsed < fade_in_duration {
        // Fade in: 0.0 → 0.7
        let t = scan.flash_elapsed / fade_in_duration;
        t * 0.7
    } else if scan.flash_elapsed < flash_duration {
        // Fade out: 0.7 → 0.0
        let t = (scan.flash_elapsed - fade_in_duration) / fade_out_duration;
        0.7 * (1.0 - t)
    } else {
        // Flash complete
        scan.flash_active = false;
        scan.flash_elapsed = 0.0;
        0.0
    };

    for (mut bg, mut vis) in &mut flash_query {
        if alpha > 0.0 {
            *vis = Visibility::Visible;
            bg.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
        } else {
            *vis = Visibility::Hidden;
            bg.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
        }
    }
}

/// System: finish scanning and start solving.
pub fn finish_scan_animation(
    mut solve: ResMut<SolveQueue>,
    mut scan: ResMut<ScanAnimation>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform), Without<Cubie>>,
    cubies_query: Query<(&Cubie, &Transform, &Children)>,
    stickers: Query<&Sticker>,
) {
    if !scan.active || !scan.returning_to_start {
        return;
    }

    // Check if return animation is complete
    if scan.elapsed < scan.duration_per_face {
        return;
    }

    // Snap camera exactly to saved orbit state
    if let Some((yaw, pitch, distance)) = scan.saved_orbit_state {
        if let Ok((mut orbit, mut cam_tf)) = camera_query.get_single_mut() {
            orbit.yaw = yaw;
            orbit.pitch = pitch;
            orbit.distance = distance;

            cam_tf.translation = Vec3::new(
                distance * pitch.cos() * yaw.sin(),
                distance * pitch.sin(),
                distance * pitch.cos() * yaw.cos(),
            );
            cam_tf.look_at(Vec3::ZERO, Vec3::Y);
        }
    }

    // Reset scan state
    *scan = ScanAnimation::default();

    // Now compute the solve moves and start solving
    let moves = compute_solve_moves(&cubies_query, &stickers);
    if !moves.is_empty() {
        solve.moves = moves;
        solve.status = SolveStatus::Active;
    } else {
        solve.status = SolveStatus::Idle;
    }
}

/// System: process the next move from the solve queue when animation is idle.
pub fn process_solve_queue(
    mut commands: Commands,
    mut solve: ResMut<SolveQueue>,
    mut animation: ResMut<FaceRotationAnimation>,
    cubies: Query<(Entity, &Cubie)>,
) {
    if solve.status != SolveStatus::Active || animation.active {
        return;
    }

    let Some(mv) = solve.moves.pop_front() else {
        return;
    };

    let affected: Vec<Entity> = cubies
        .iter()
        .filter(|(_, c)| mv.axis.layer(c.grid_position) == mv.layer)
        .map(|(e, _)| e)
        .collect();

    if affected.is_empty() {
        return;
    }

    let pivot = commands
        .spawn((
            RotationPivot,
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    for &entity in &affected {
        commands.entity(entity).set_parent(pivot);
    }

    let angle = if mv.clockwise {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    };

    *animation = FaceRotationAnimation {
        active: true,
        pivot_entity: Some(pivot),
        affected_cubies: affected,
        rotation_axis: mv.axis.to_vec3(),
        target_angle: angle,
        current_angle: 0.0,
        duration: 0.3,
        elapsed: 0.0,
        move_data: mv,
        origin: ActionOrigin::Solve,
    };
}

/// System: after the last solve move completes, return to idle.
/// Unlike scramble, solve-moves are kept in ActionHistory so the user can undo.
pub fn finish_solve(
    mut solve: ResMut<SolveQueue>,
    animation: Res<FaceRotationAnimation>,
) {
    if solve.status != SolveStatus::Active {
        return;
    }

    if !solve.moves.is_empty() || animation.active {
        return;
    }

    solve.status = SolveStatus::Idle;
}

/// System: update Solve button appearance based on state.
pub fn update_solve_button(
    solve: Res<SolveQueue>,
    scramble: Res<ScrambleQueue>,
    animation: Res<FaceRotationAnimation>,
    mut button_query: Query<(&Children, &mut BackgroundColor), With<SolveButton>>,
    mut text_query: Query<&mut TextColor>,
) {
    let enabled = solve.status == SolveStatus::Idle
        && scramble.status == ScrambleStatus::Idle
        && !animation.active;

    let active_bg = Color::srgba(0.2, 0.25, 0.2, 0.8);
    let inactive_bg = Color::srgba(0.2, 0.25, 0.2, 0.4);
    let active_text = Color::srgba(1.0, 1.0, 1.0, 0.9);
    let inactive_text = Color::srgba(1.0, 1.0, 1.0, 0.4);

    for (children, mut bg) in button_query.iter_mut() {
        *bg = BackgroundColor(if enabled { active_bg } else { inactive_bg });
        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(if enabled { active_text } else { inactive_text });
            }
        }
    }
}
