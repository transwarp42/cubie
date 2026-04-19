use bevy::prelude::*;

use super::model::*;

const CUBIE_HALF: f32 = 0.45; // CUBIE_SIZE / 2

/// Result of a raycast hit on a cubie face.
#[derive(Debug, Clone, Copy)]
pub struct CubieHit {
    pub cubie_entity: Entity,
    pub grid_position: IVec3,
    pub face_direction: FaceDirection,
    pub world_position: Vec3,
    pub face_normal: Vec3,
}

/// Cast a ray from the mouse cursor and find the nearest cubie face hit.
pub fn raycast_cubies(
    cursor_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cubies: &[(Entity, IVec3, GlobalTransform)],
) -> Option<CubieHit> {
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return None;
    };

    let mut best: Option<(f32, CubieHit)> = None;

    // Six face normals
    let face_normals = [
        (Vec3::X, FaceDirection::Right),
        (Vec3::NEG_X, FaceDirection::Left),
        (Vec3::Y, FaceDirection::Up),
        (Vec3::NEG_Y, FaceDirection::Down),
        (Vec3::Z, FaceDirection::Front),
        (Vec3::NEG_Z, FaceDirection::Back),
    ];

    for &(entity, grid_pos, ref global_tf) in cubies {
        let center = global_tf.translation();

        for &(normal, face_dir) in &face_normals {
            // Only test faces that are on the outer surface of the cube
            let grid_component = match face_dir {
                FaceDirection::Right | FaceDirection::Left => grid_pos.x,
                FaceDirection::Up | FaceDirection::Down => grid_pos.y,
                FaceDirection::Front | FaceDirection::Back => grid_pos.z,
            };
            let is_outer = match face_dir {
                FaceDirection::Right | FaceDirection::Up | FaceDirection::Front => grid_component == 1,
                FaceDirection::Left | FaceDirection::Down | FaceDirection::Back => grid_component == -1,
            };
            if !is_outer {
                continue;
            }

            let plane_point = center + normal * CUBIE_HALF;
            let denom = ray.direction.dot(normal);
            if denom.abs() < 1e-6 {
                continue; // Ray parallel to face
            }

            let t = (plane_point - ray.origin).dot(normal) / denom;
            if t < 0.0 {
                continue; // Behind camera
            }

            let hit_point = ray.origin + *ray.direction * t;
            let local = hit_point - center;

            // Check if hit point is within the face bounds
            let (u, v) = match face_dir {
                FaceDirection::Right | FaceDirection::Left => (local.z, local.y),
                FaceDirection::Up | FaceDirection::Down => (local.x, local.z),
                FaceDirection::Front | FaceDirection::Back => (local.x, local.y),
            };

            if u.abs() <= CUBIE_HALF && v.abs() <= CUBIE_HALF {
                if best.is_none() || t < best.unwrap().0 {
                    best = Some((t, CubieHit {
                        cubie_entity: entity,
                        grid_position: grid_pos,
                        face_direction: face_dir,
                        world_position: hit_point,
                        face_normal: normal,
                    }));
                }
            }
        }
    }

    best.map(|(_, hit)| hit)
}

