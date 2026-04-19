use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::model::*;
use super::picking::*;

/// Drag state machine phases.
#[derive(Debug, Clone)]
pub enum DragPhase {
    /// No interaction.
    Idle,
    /// Mouse pressed on a cubie, waiting for drag threshold.
    Pending {
        hit: CubieHit,
        start_screen_pos: Vec2,
    },
    /// Drag direction resolved, rotation should start.
    Resolved {
        hit: CubieHit,
        axis: RotationAxis,
        clockwise: bool,
    },
    /// Rotation animation is playing.
    Animating,
}

/// Resource tracking the current drag/interaction state.
#[derive(Resource)]
pub struct DragState {
    pub phase: DragPhase,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            phase: DragPhase::Idle,
        }
    }
}

const DRAG_THRESHOLD: f32 = 10.0;

/// System: handle mouse input and manage the drag state machine.
pub fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<super::super::camera::OrbitCamera>>,
    cubies: Query<(Entity, &Cubie, &GlobalTransform)>,
    mut drag_state: ResMut<DragState>,
) {
    // Don't process input during animation
    if matches!(drag_state.phase, DragPhase::Animating) {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, cam_tf)) = camera_q.get_single() else { return };

    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let cubie_list: Vec<_> = cubies
                .iter()
                .map(|(e, c, gt)| (e, c.grid_position, gt.clone()))
                .collect();

            if let Some(hit) = raycast_cubies(cursor_pos, camera, cam_tf, &cubie_list) {
                drag_state.phase = DragPhase::Pending {
                    hit,
                    start_screen_pos: cursor_pos,
                };
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        if !matches!(drag_state.phase, DragPhase::Animating) {
            // If we're in Resolved, don't reset — the rotation system will handle it.
            if !matches!(drag_state.phase, DragPhase::Resolved { .. }) {
                drag_state.phase = DragPhase::Idle;
            }
        }
    }
}

/// System: once dragging past threshold, determine rotation axis and direction.
pub fn resolve_drag_direction(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<super::super::camera::OrbitCamera>>,
    mut drag_state: ResMut<DragState>,
) {
    let DragPhase::Pending { hit, start_screen_pos } = drag_state.phase else {
        return;
    };

    let Ok(window) = windows.get_single() else { return };
    let Some(current_pos) = window.cursor_position() else { return };
    let Ok((_, cam_tf)) = camera_q.get_single() else { return };

    let screen_delta = current_pos - start_screen_pos;
    if screen_delta.length() < DRAG_THRESHOLD {
        return;
    }

    // Convert screen-space drag to world-space
    let cam_right = cam_tf.right().as_vec3();
    let cam_up = cam_tf.up().as_vec3();
    let world_drag = screen_delta.x * cam_right + (-screen_delta.y) * cam_up;

    // Project onto face plane (remove component along face normal)
    let n = hit.face_normal;
    let projected = world_drag - world_drag.dot(n) * n;

    if projected.length() < 1e-6 {
        return;
    }

    // Cross product: rotation axis = drag_direction × face_normal
    let cross = projected.normalize().cross(n);

    // Snap to nearest main axis
    let abs = cross.abs();
    let (axis, component) = if abs.x >= abs.y && abs.x >= abs.z {
        (RotationAxis::X, cross.x)
    } else if abs.y >= abs.x && abs.y >= abs.z {
        (RotationAxis::Y, cross.y)
    } else {
        (RotationAxis::Z, cross.z)
    };

    let clockwise = component > 0.0;

    drag_state.phase = DragPhase::Resolved {
        hit,
        axis,
        clockwise,
    };
}

