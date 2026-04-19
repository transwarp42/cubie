use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use crate::cube::input::{DragPhase, DragState};

/// Orbit camera component for rotating around the cube.
#[derive(Component)]
pub struct OrbitCamera {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            distance: 8.0,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: std::f32::consts::FRAC_PI_6,
            sensitivity: 0.005,
        }
    }
}

/// Calculate camera position from spherical coordinates.
fn orbit_position(orbit: &OrbitCamera) -> Vec3 {
    Vec3::new(
        orbit.distance * orbit.pitch.cos() * orbit.yaw.sin(),
        orbit.distance * orbit.pitch.sin(),
        orbit.distance * orbit.pitch.cos() * orbit.yaw.cos(),
    )
}

/// Setup system: spawn camera and lighting.
pub fn setup_camera(mut commands: Commands) {
    let orbit = OrbitCamera::default();
    let pos = orbit_position(&orbit);

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y),
        orbit,
    ));

    // Directional light for depth effect
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
    ));
}

/// Update system: process mouse input to rotate and zoom the camera.
pub fn orbit_camera_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    drag_state: Res<DragState>,
) {
    let Ok((mut orbit, mut transform)) = query.get_single_mut() else {
        return;
    };

    // Only allow orbit when no cube interaction is happening
    let allow_orbit = matches!(drag_state.phase, DragPhase::Idle);

    // Rotate when left mouse button is pressed
    if allow_orbit && mouse_button.pressed(MouseButton::Left) {
        for event in mouse_motion.read() {
            orbit.yaw -= event.delta.x * orbit.sensitivity;
            orbit.pitch += event.delta.y * orbit.sensitivity;
            orbit.pitch = orbit.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.05,
                std::f32::consts::FRAC_PI_2 - 0.05,
            );
        }
    } else {
        mouse_motion.clear();
    }

    // Zoom with scroll wheel
    for event in scroll.read() {
        orbit.distance -= event.y * 0.5;
        orbit.distance = orbit.distance.clamp(4.0, 15.0);
    }

    // Update camera position
    transform.translation = orbit_position(&orbit);
    transform.look_at(Vec3::ZERO, Vec3::Y);
}
