use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::cube::input::{DragPhase, DragState};
use crate::cube::solver::{SolveQueue, SolveStatus};

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
        Tonemapping::None,
        Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y),
        orbit,
    ));
}

/// Update system: process mouse input to rotate and zoom the camera.
pub fn orbit_camera_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    drag_state: Res<DragState>,
    solve_queue: Res<SolveQueue>,
) {
    let Ok((mut orbit, mut transform)) = query.get_single_mut() else {
        return;
    };

    // Block orbit updates during scanning
    if solve_queue.status == SolveStatus::Scanning {
        mouse_motion.clear();
        return;
    }

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


    // Update camera position
    transform.translation = orbit_position(&orbit);
    transform.look_at(Vec3::ZERO, Vec3::Y);
}
