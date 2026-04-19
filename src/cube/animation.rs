use bevy::prelude::*;

use super::model::*;

/// Tracks whether a rotation was triggered by user drag, undo, or redo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActionOrigin {
    #[default]
    Regular,
    Undo,
    Redo,
}

/// Ease-out cubic: 1 - (1 - t)^3
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Resource tracking the active face rotation animation.
#[derive(Resource)]
pub struct FaceRotationAnimation {
    pub active: bool,
    pub pivot_entity: Option<Entity>,
    pub affected_cubies: Vec<Entity>,
    pub rotation_axis: Vec3,
    pub target_angle: f32,
    pub current_angle: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub move_data: CubeMove,
    pub origin: ActionOrigin,
}

impl Default for FaceRotationAnimation {
    fn default() -> Self {
        Self {
            active: false,
            pivot_entity: None,
            affected_cubies: Vec::new(),
            rotation_axis: Vec3::Y,
            target_angle: 0.0,
            current_angle: 0.0,
            duration: 0.3,
            elapsed: 0.0,
            move_data: CubeMove {
                axis: RotationAxis::Y,
                layer: 0,
                clockwise: true,
            },
            origin: ActionOrigin::Regular,
        }
    }
}

/// System: animate the pivot rotation each frame.
pub fn animate_face_rotation(
    time: Res<Time>,
    mut animation: ResMut<FaceRotationAnimation>,
    mut transforms: Query<&mut Transform>,
) {
    if !animation.active {
        return;
    }

    let Some(pivot) = animation.pivot_entity else {
        return;
    };

    animation.elapsed += time.delta_secs();
    let t = (animation.elapsed / animation.duration).min(1.0);
    let t_eased = ease_out_cubic(t);

    let new_angle = animation.target_angle * t_eased;

    if let Ok(mut pivot_tf) = transforms.get_mut(pivot) {
        pivot_tf.rotation = Quat::from_axis_angle(animation.rotation_axis, new_angle);
    }

    animation.current_angle = new_angle;
}

