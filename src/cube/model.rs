use bevy::prelude::*;

/// Direction of a face on a cubie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceDirection {
    Up,    // +Y
    Down,  // -Y
    Right, // +X
    Left,  // -X
    Front, // +Z
    Back,  // -Z
}

impl FaceDirection {
    /// Return the world-space normal for this face direction.
    pub fn normal(self) -> Vec3 {
        match self {
            FaceDirection::Up => Vec3::Y,
            FaceDirection::Down => Vec3::NEG_Y,
            FaceDirection::Right => Vec3::X,
            FaceDirection::Left => Vec3::NEG_X,
            FaceDirection::Front => Vec3::Z,
            FaceDirection::Back => Vec3::NEG_Z,
        }
    }

    /// Determine face direction from a world-space normal vector.
    pub fn from_normal(n: Vec3) -> Self {
        let abs = n.abs();
        if abs.x >= abs.y && abs.x >= abs.z {
            if n.x > 0.0 { FaceDirection::Right } else { FaceDirection::Left }
        } else if abs.y >= abs.x && abs.y >= abs.z {
            if n.y > 0.0 { FaceDirection::Up } else { FaceDirection::Down }
        } else {
            if n.z > 0.0 { FaceDirection::Front } else { FaceDirection::Back }
        }
    }

    /// Rotate this face direction 90° clockwise around the given axis
    /// (clockwise when looking from the positive end of the axis).
    pub fn rotated_cw(self, axis: RotationAxis) -> Self {
        match axis {
            RotationAxis::X => match self {
                FaceDirection::Front => FaceDirection::Up,
                FaceDirection::Up => FaceDirection::Back,
                FaceDirection::Back => FaceDirection::Down,
                FaceDirection::Down => FaceDirection::Front,
                other => other,
            },
            RotationAxis::Y => match self {
                FaceDirection::Front => FaceDirection::Right,
                FaceDirection::Right => FaceDirection::Back,
                FaceDirection::Back => FaceDirection::Left,
                FaceDirection::Left => FaceDirection::Front,
                other => other,
            },
            RotationAxis::Z => match self {
                FaceDirection::Up => FaceDirection::Right,
                FaceDirection::Right => FaceDirection::Down,
                FaceDirection::Down => FaceDirection::Left,
                FaceDirection::Left => FaceDirection::Up,
                other => other,
            },
        }
    }

    /// Rotate this face direction 90° counter-clockwise around the given axis.
    pub fn rotated_ccw(self, axis: RotationAxis) -> Self {
        // CCW = 3x CW
        self.rotated_cw(axis).rotated_cw(axis).rotated_cw(axis)
    }
}

/// The three main rotation axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationAxis {
    X,
    Y,
    Z,
}

impl RotationAxis {
    pub fn to_vec3(self) -> Vec3 {
        match self {
            RotationAxis::X => Vec3::X,
            RotationAxis::Y => Vec3::Y,
            RotationAxis::Z => Vec3::Z,
        }
    }

    /// Extract the relevant coordinate from grid_position for this axis.
    pub fn layer(self, pos: IVec3) -> i32 {
        match self {
            RotationAxis::X => pos.x,
            RotationAxis::Y => pos.y,
            RotationAxis::Z => pos.z,
        }
    }
}

/// Color of a sticker on the Rubik's Cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StickerColor {
    White,
    Yellow,
    Red,
    Orange,
    Blue,
    Green,
}

impl StickerColor {
    /// Convert the sticker color to a Bevy `Color`.
    pub fn to_color(self) -> Color {
        match self {
            StickerColor::White  => Color::srgb(0.95, 0.95, 0.95),
            StickerColor::Yellow => Color::srgb(1.0, 0.85, 0.0),
            StickerColor::Red    => Color::srgb(0.8, 0.05, 0.05),
            StickerColor::Orange => Color::srgb(1.0, 0.5, 0.0),
            StickerColor::Blue   => Color::srgb(0.0, 0.1, 0.75),
            StickerColor::Green  => Color::srgb(0.0, 0.6, 0.1),
        }
    }
}

/// Component attached to sticker entities.
#[derive(Component)]
pub struct Sticker {
    pub face_direction: FaceDirection,
    pub color: StickerColor,
}

/// Logical data for a single cubie.
pub struct CubieData {
    pub grid_position: IVec3,
    pub stickers: Vec<(FaceDirection, StickerColor)>,
}

/// Component linking a cubie entity to the logical model.
#[derive(Component)]
pub struct Cubie {
    pub grid_position: IVec3,
}

/// A single move on the cube.
#[derive(Debug, Clone, Copy)]
pub struct CubeMove {
    pub axis: RotationAxis,
    pub layer: i32,
    pub clockwise: bool,
}

/// The complete logical state of the Rubik's Cube.
#[derive(Resource)]
pub struct CubeState {
    pub cubies: Vec<CubieData>,
}

impl CubeState {
    /// Create a solved 3×3 cube.
    pub fn solved() -> Self {
        let mut cubies = Vec::new();

        for x in -1..=1i32 {
            for y in -1..=1i32 {
                for z in -1..=1i32 {
                    let pos = IVec3::new(x, y, z);
                    let mut stickers = Vec::new();

                    if x == 1  { stickers.push((FaceDirection::Right, StickerColor::Red)); }
                    if x == -1 { stickers.push((FaceDirection::Left, StickerColor::Orange)); }
                    if y == 1  { stickers.push((FaceDirection::Up, StickerColor::White)); }
                    if y == -1 { stickers.push((FaceDirection::Down, StickerColor::Yellow)); }
                    if z == 1  { stickers.push((FaceDirection::Front, StickerColor::Green)); }
                    if z == -1 { stickers.push((FaceDirection::Back, StickerColor::Blue)); }

                    cubies.push(CubieData {
                        grid_position: pos,
                        stickers,
                    });
                }
            }
        }

        CubeState { cubies }
    }

    /// Apply a 90° rotation to the logical state.
    pub fn apply_rotation(&mut self, mv: CubeMove) {
        for cubie in &mut self.cubies {
            if mv.axis.layer(cubie.grid_position) != mv.layer {
                continue;
            }

            // Rotate grid position
            let p = cubie.grid_position;
            cubie.grid_position = if mv.clockwise {
                match mv.axis {
                    RotationAxis::X => IVec3::new(p.x, -p.z, p.y),
                    RotationAxis::Y => IVec3::new(p.z, p.y, -p.x),
                    RotationAxis::Z => IVec3::new(p.y, -p.x, p.z),
                }
            } else {
                match mv.axis {
                    RotationAxis::X => IVec3::new(p.x, p.z, -p.y),
                    RotationAxis::Y => IVec3::new(-p.z, p.y, p.x),
                    RotationAxis::Z => IVec3::new(-p.y, p.x, p.z),
                }
            };

            // Rotate face directions of stickers
            for (face_dir, _) in &mut cubie.stickers {
                *face_dir = if mv.clockwise {
                    face_dir.rotated_cw(mv.axis)
                } else {
                    face_dir.rotated_ccw(mv.axis)
                };
            }
        }
    }
}
