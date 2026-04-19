mod camera;
mod cube;
mod icon;

use bevy::prelude::*;
use cube::model::CubeState;
use cube::input::DragState;
use cube::animation::FaceRotationAnimation;
use cube::labels::FaceLabelsVisible;
use cube::history::ActionHistory;
use cube::scramble::{ScrambleQueue, ResetCubeEvent};

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
        .insert_resource(CubeState::solved())
        .insert_resource(DragState::default())
        .insert_resource(FaceRotationAnimation::default())
        .insert_resource(FaceLabelsVisible::default())
        .insert_resource(ActionHistory::default())
        .insert_resource(ScrambleQueue::default())
        .add_event::<ResetCubeEvent>()
        .add_systems(Startup, (camera::setup_camera, cube::spawn::spawn_cube, cube::labels::spawn_face_labels, cube::history::spawn_undo_redo_buttons, cube::scramble::spawn_scramble_button, icon::set_app_icon))
        .add_systems(Update, (
            cube::scramble::handle_scramble_input,
            cube::scramble::handle_reset_input,
            cube::scramble::handle_scramble_confirmation,
            cube::input::handle_mouse_input,
            cube::input::resolve_drag_direction,
            cube::history::handle_undo_redo_input,
            cube::rotation::start_face_rotation,
            cube::scramble::process_scramble_queue,
            cube::animation::animate_face_rotation,
            cube::rotation::finish_face_rotation,
            cube::scramble::finish_scramble,
            cube::scramble::execute_reset,
            camera::orbit_camera_system,
            cube::labels::update_face_labels,
            cube::labels::toggle_labels_button,
            cube::history::update_undo_redo_buttons,
            cube::scramble::update_scramble_button,
        ).chain());

    app.run();
}
