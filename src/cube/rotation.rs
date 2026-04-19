use bevy::prelude::*;

use super::model::*;
use super::input::{DragPhase, DragState};
use super::animation::{ActionOrigin, FaceRotationAnimation};
use super::history::ActionHistory;

/// Marker for the temporary pivot entity used during rotation animation.
#[derive(Component)]
pub struct RotationPivot;

/// All 24 valid rotation quaternions for a cube (multiples of 90° around principal axes).
/// Generated from the 6 possible directions for the X-axis × 4 rotations around it.
fn all_24_orientations() -> [Quat; 24] {
    use std::f32::consts::FRAC_PI_2;

    // The 6 face rotations that place each axis direction as the new +X
    let face_rotations = [
        Quat::IDENTITY,                              // +X stays +X
        Quat::from_rotation_y(FRAC_PI_2),            // +Z becomes +X
        Quat::from_rotation_y(std::f32::consts::PI), // -X becomes +X
        Quat::from_rotation_y(-FRAC_PI_2),           // -Z becomes +X
        Quat::from_rotation_z(FRAC_PI_2),            // +Y becomes +X
        Quat::from_rotation_z(-FRAC_PI_2),           // -Y becomes +X
    ];

    // 4 rotations around the X-axis (0°, 90°, 180°, 270°)
    let x_rotations = [
        Quat::IDENTITY,
        Quat::from_rotation_x(FRAC_PI_2),
        Quat::from_rotation_x(std::f32::consts::PI),
        Quat::from_rotation_x(-FRAC_PI_2),
    ];

    let mut orientations = [Quat::IDENTITY; 24];
    let mut i = 0;
    for face in &face_rotations {
        for around in &x_rotations {
            orientations[i] = (*face * *around).normalize();
            i += 1;
        }
    }
    orientations
}

/// Snap a quaternion to the nearest of the 24 valid axis-aligned cube orientations.
/// Uses quaternion dot product to find the closest match, guaranteeing a valid result.
fn snap_rotation(q: Quat) -> Quat {
    let orientations = all_24_orientations();
    let mut best = Quat::IDENTITY;
    let mut best_dot: f32 = -1.0;

    for &candidate in &orientations {
        // |dot| because q and -q represent the same rotation
        let dot = q.dot(candidate).abs();
        if dot > best_dot {
            best_dot = dot;
            best = candidate;
        }
    }
    best
}

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
                // Snap rotation to nearest axis-aligned orientation to prevent
                // floating-point drift accumulating over many moves (especially scramble).
                transform.rotation = snap_rotation(gt_transform.rotation);

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

