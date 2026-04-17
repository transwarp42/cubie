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

/// Logical data for a single cubie.
pub struct CubieData {
    pub grid_position: IVec3,
    pub stickers: Vec<(FaceDirection, StickerColor)>,
}

/// Component linking a cubie entity to the logical model.
#[derive(Component)]
pub struct Cubie {
    /// Will be used later for layer rotations.
    #[allow(dead_code)]
    pub grid_position: IVec3,
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
}
