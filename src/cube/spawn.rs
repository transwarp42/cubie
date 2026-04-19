use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};

use super::model::*;

const CUBIE_SIZE: f32 = 0.9;
const STICKER_SIZE: f32 = 0.82;
const STICKER_ELEVATION: f32 = 0.001;

/// Calculate the local transform of a sticker relative to the cubie center.
fn sticker_transform(dir: FaceDirection) -> Transform {
    let offset = CUBIE_SIZE / 2.0 + STICKER_ELEVATION;
    match dir {
        FaceDirection::Right => Transform::from_xyz(offset, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        FaceDirection::Left => Transform::from_xyz(-offset, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        FaceDirection::Up => Transform::from_xyz(0.0, offset, 0.0)
            .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        FaceDirection::Down => Transform::from_xyz(0.0, -offset, 0.0)
            .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        FaceDirection::Front => Transform::from_xyz(0.0, 0.0, offset),
        FaceDirection::Back => Transform::from_xyz(0.0, 0.0, -offset)
            .with_rotation(Quat::from_rotation_y(PI)),
    }
}

/// Spawn all cubie entities based on the CubeState.
pub fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_state: Res<CubeState>,
) {
    // Shared meshes
    let body_mesh = meshes.add(Cuboid::new(CUBIE_SIZE, CUBIE_SIZE, CUBIE_SIZE));
    let sticker_mesh = meshes.add(Rectangle::new(STICKER_SIZE, STICKER_SIZE));

    // Material for the black cubie body
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.05, 0.05),
        ..default()
    });

    // Pre-create materials per sticker color — unlit for consistent colors
    let mut color_materials: HashMap<StickerColor, Handle<StandardMaterial>> = HashMap::new();
    for &color in &[
        StickerColor::White,
        StickerColor::Yellow,
        StickerColor::Red,
        StickerColor::Orange,
        StickerColor::Blue,
        StickerColor::Green,
    ] {
        color_materials.insert(
            color,
            materials.add(StandardMaterial {
                base_color: color.to_color(),
                unlit: true,
                ..default()
            }),
        );
    }

    // Spawn each cubie
    for cubie_data in &cube_state.cubies {
        let pos = cubie_data.grid_position;
        let world_pos = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);

        commands
            .spawn((
                Cubie { grid_position: pos },
                Transform::from_translation(world_pos),
                Visibility::default(),
            ))
            .with_children(|parent| {
                // Black cubie body
                parent.spawn((
                    Mesh3d(body_mesh.clone()),
                    MeshMaterial3d(body_material.clone()),
                ));

                // Colored stickers on outer faces
                for &(face_dir, color) in &cubie_data.stickers {
                    parent.spawn((
                        Sticker {
                            face_direction: face_dir,
                            color,
                        },
                        Mesh3d(sticker_mesh.clone()),
                        MeshMaterial3d(color_materials[&color].clone()),
                        sticker_transform(face_dir),
                    ));
                }
            });
    }
}
