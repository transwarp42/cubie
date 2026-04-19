use bevy::prelude::*;

use super::model::FaceDirection;

const LABEL_OFFSET: f32 = 2.5;

/// Resource to track whether face labels are visible.
#[derive(Resource)]
pub struct FaceLabelsVisible(pub bool);

impl Default for FaceLabelsVisible {
    fn default() -> Self {
        Self(true)
    }
}

/// Marker for the face label UI nodes.
#[derive(Component)]
pub struct FaceLabel {
    pub world_position: Vec3,
}

/// Marker for the toggle button.
#[derive(Component)]
pub struct ToggleLabelsButton;

/// Spawn UI text nodes for each cube face and a toggle button.
pub fn spawn_face_labels(mut commands: Commands) {
    let faces: [(FaceDirection, &str, Color); 6] = [
        (FaceDirection::Front, "Front", Color::srgba(0.0, 0.85, 0.15, 0.9)),
        (FaceDirection::Back, "Back", Color::srgba(0.2, 0.4, 1.0, 0.9)),
        (FaceDirection::Up, "Top", Color::srgba(0.95, 0.95, 0.95, 0.9)),
        (FaceDirection::Down, "Bottom", Color::srgba(1.0, 0.85, 0.0, 0.9)),
        (FaceDirection::Right, "Right", Color::srgba(0.9, 0.15, 0.15, 0.9)),
        (FaceDirection::Left, "Left", Color::srgba(1.0, 0.55, 0.05, 0.9)),
    ];

    for (dir, name, color) in faces {
        let world_pos = dir.normal() * LABEL_OFFSET;

        commands.spawn((
            FaceLabel { world_position: world_pos },
            Text::new(name),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(color),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        ));
    }

    // Toggle button in top-right corner
    commands
        .spawn((
            ToggleLabelsButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Labels: On"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ));
        });
}

/// Handle toggle button clicks.
pub fn toggle_labels_button(
    mut interaction_query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<ToggleLabelsButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut labels_visible: ResMut<FaceLabelsVisible>,
) {
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            labels_visible.0 = !labels_visible.0;
            // Update button text
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = if labels_visible.0 {
                        "Labels: On".to_string()
                    } else {
                        "Labels: Off".to_string()
                    };
                }
            }
        }
    }
}

/// Project face labels from 3D world positions to 2D screen positions.
pub fn update_face_labels(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut label_query: Query<(&FaceLabel, &mut Node, &mut Visibility)>,
    labels_visible: Res<FaceLabelsVisible>,
) {
    let Ok((camera, camera_gt)) = camera_query.get_single() else {
        return;
    };

    if !labels_visible.0 {
        for (_, _, mut visibility) in &mut label_query {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let cam_pos = camera_gt.translation();

    for (label, mut node, mut visibility) in &mut label_query {
        let face_normal = label.world_position.normalize();
        let to_camera = (cam_pos - label.world_position).normalize();
        let dot = face_normal.dot(to_camera);

        if dot < 0.1 {
            *visibility = Visibility::Hidden;
            continue;
        }

        if let Ok(screen_pos) = camera.world_to_viewport(camera_gt, label.world_position) {
            *visibility = Visibility::Visible;
            node.left = Val::Px(screen_pos.x - 25.0);
            node.top = Val::Px(screen_pos.y - 10.0);
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
