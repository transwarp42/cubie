mod camera;
mod cube;
mod icon;

use bevy::prelude::*;
use cube::model::CubeState;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cubie — Rubik's Cube".into(),
                resolution: (900.0, 700.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.2)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
        })
        .insert_resource(CubeState::solved())
        .add_systems(Startup, (camera::setup_camera, cube::spawn::spawn_cube, icon::set_app_icon))
        .add_systems(Update, camera::orbit_camera_system);

    app.run();
}
