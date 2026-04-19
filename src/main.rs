mod camera;
mod cube;
mod icon;

use bevy::prelude::*;
use cube::model::CubeState;
use cube::input::DragState;
use cube::animation::FaceRotationAnimation;
use cube::labels::FaceLabelsVisible;
use cube::history::ActionHistory;

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
        .insert_resource(DragState::default())
        .insert_resource(FaceRotationAnimation::default())
        .insert_resource(FaceLabelsVisible::default())
        .insert_resource(ActionHistory::default())
        .add_systems(Startup, (camera::setup_camera, cube::spawn::spawn_cube, cube::labels::spawn_face_labels, cube::history::spawn_undo_redo_buttons, icon::set_app_icon))
        .add_systems(Update, (
            cube::input::handle_mouse_input,
            cube::input::resolve_drag_direction,
            cube::history::handle_undo_redo_input,
            cube::rotation::start_face_rotation,
            cube::animation::animate_face_rotation,
            cube::rotation::finish_face_rotation,
            camera::orbit_camera_system,
            cube::labels::update_face_labels,
            cube::labels::toggle_labels_button,
            cube::history::update_undo_redo_buttons,
        ).chain());

    app.run();
}
