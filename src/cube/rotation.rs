use bevy::prelude::*;

use super::model::*;
use super::input::{DragPhase, DragState};
use super::animation::{ActionOrigin, FaceRotationAnimation};
use super::history::ActionHistory;

/// Marker for the temporary pivot entity used during rotation animation.
#[derive(Component)]
pub struct RotationPivot;

/// System: when drag is resolved, start the face rotation animation.
pub fn start_face_rotation(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mut animation: ResMut<FaceRotationAnimation>,
    cubies: Query<(Entity, &Cubie)>,
) {
    let DragPhase::Resolved { hit, axis, clockwise } = drag_state.phase else {
        return;
    };

    if animation.active {
        return;
    }

    // Determine which layer to rotate
    let layer = axis.layer(hit.grid_position);

    // Collect the 9 cubie entities in this slice
    let affected: Vec<Entity> = cubies
        .iter()
        .filter(|(_, c)| axis.layer(c.grid_position) == layer)
        .map(|(e, _)| e)
        .collect();

    if affected.is_empty() {
        drag_state.phase = DragPhase::Idle;
        return;
    }

    // Create pivot entity at origin
    let pivot = commands
        .spawn((
            RotationPivot,
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    // Reparent affected cubies under the pivot
    for &entity in &affected {
        commands.entity(entity).set_parent(pivot);
    }

    let angle = if clockwise {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    };

    *animation = FaceRotationAnimation {
        active: true,
        pivot_entity: Some(pivot),
        affected_cubies: affected,
        rotation_axis: axis.to_vec3(),
        target_angle: angle,
        current_angle: 0.0,
        duration: 0.3,
        elapsed: 0.0,
        move_data: CubeMove { axis, layer, clockwise },
        origin: ActionOrigin::Regular,
    };

    drag_state.phase = DragPhase::Animating;
}

/// System: after animation completes, finalize transforms and update logical state.
pub fn finish_face_rotation(
    mut commands: Commands,
    mut animation: ResMut<FaceRotationAnimation>,
    mut drag_state: ResMut<DragState>,
    mut cube_state: ResMut<CubeState>,
    mut history: ResMut<ActionHistory>,
    mut cubies: Query<(&mut Cubie, &mut Transform)>,
    global_transforms: Query<&GlobalTransform>,
) {
    if !animation.active || animation.elapsed < animation.duration {
        return;
    }

    let mv = animation.move_data;

    // Read final global transforms, deparent, and set local transforms
    for &entity in &animation.affected_cubies {
        if let Ok(gt) = global_transforms.get(entity) {
            let gt_transform = gt.compute_transform();
            // Snap position to grid
            let snapped = Vec3::new(
                gt_transform.translation.x.round(),
                gt_transform.translation.y.round(),
                gt_transform.translation.z.round(),
            );

            commands.entity(entity).remove_parent();

            if let Ok((mut cubie, mut transform)) = cubies.get_mut(entity) {
                transform.translation = snapped;
                transform.rotation = gt_transform.rotation;
                // Normalize to avoid drift
                transform.rotation = transform.rotation.normalize();

                // Update grid position on the component
                cubie.grid_position = IVec3::new(
                    snapped.x as i32,
                    snapped.y as i32,
                    snapped.z as i32,
                );
            }
        }
    }

    // Remove pivot entity
    if let Some(pivot) = animation.pivot_entity {
        commands.entity(pivot).despawn();
    }

    // Update logical cube state
    cube_state.apply_rotation(mv);

    // Update undo/redo stacks based on action origin
    match animation.origin {
        ActionOrigin::Regular => {
            history.push_action(mv);
        }
        ActionOrigin::Undo | ActionOrigin::Redo | ActionOrigin::Scramble => {
            // Already handled elsewhere
        }
    }

    // Reset
    *animation = FaceRotationAnimation::default();
    drag_state.phase = DragPhase::Idle;
}

