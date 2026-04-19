use std::collections::VecDeque;

use bevy::prelude::*;
use rand::Rng;

use super::animation::{ActionOrigin, FaceRotationAnimation};
use super::history::ActionHistory;
use super::model::*;
use super::rotation::RotationPivot;

/// Status of the scramble state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrambleStatus {
    #[default]
    Idle,
    Confirming,
    Active,
}

/// Resource tracking the scramble queue and status.
#[derive(Resource, Default)]
pub struct ScrambleQueue {
    pub status: ScrambleStatus,
    pub moves: VecDeque<CubeMove>,
}

impl ScrambleQueue {
    pub fn is_active(&self) -> bool {
        self.status == ScrambleStatus::Active
    }
}

/// Marker for the scramble button.
#[derive(Component)]
pub struct ScrambleButton;

/// Marker for the confirmation dialog overlay.
#[derive(Component)]
pub struct ScrambleConfirmDialog;

/// Marker for the confirm (yes) button.
#[derive(Component)]
pub struct ScrambleConfirmYes;

/// Marker for the cancel (no) button.
#[derive(Component)]
pub struct ScrambleConfirmNo;

/// Generate a random-move scramble of 20–25 moves.
fn generate_scramble_moves() -> VecDeque<CubeMove> {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(20..=25);
    let mut moves = VecDeque::with_capacity(count * 2); // extra capacity for 180° doubles

    let axes = [RotationAxis::X, RotationAxis::Y, RotationAxis::Z];
    let layers = [-1i32, 1];

    let mut last_axis: Option<RotationAxis> = None;
    let mut last_layer: Option<i32> = None;

    for _ in 0..count {
        loop {
            let axis = axes[rng.gen_range(0..3)];
            let layer = layers[rng.gen_range(0..2)];

            // Avoid repeating same axis+layer
            if last_axis == Some(axis) && last_layer == Some(layer) {
                continue;
            }

            let rotation_type = rng.gen_range(0..3); // 0 = 90° CW, 1 = 90° CCW, 2 = 180°
            let clockwise = rotation_type == 0;
            let mv = CubeMove { axis, layer, clockwise };

            if rotation_type == 2 {
                // 180°: add the same move twice
                let mv_180 = CubeMove { axis, layer, clockwise: true };
                moves.push_back(mv_180);
                moves.push_back(mv_180);
            } else {
                moves.push_back(mv);
            }

            last_axis = Some(axis);
            last_layer = Some(layer);
            break;
        }
    }

    moves
}

/// Spawn the scramble button (left of Labels, right of Undo/Redo).
pub fn spawn_scramble_button(mut commands: Commands) {
    commands
        .spawn((
            ScrambleButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(230.0),
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
                Text::new("Scramble"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ));
        });
}

/// System: handle scramble button click → show confirmation dialog.
pub fn handle_scramble_input(
    mut commands: Commands,
    mut scramble: ResMut<ScrambleQueue>,
    animation: Res<FaceRotationAnimation>,
    query: Query<&Interaction, (Changed<Interaction>, With<ScrambleButton>)>,
) {
    if scramble.status != ScrambleStatus::Idle || animation.active {
        return;
    }

    for interaction in &query {
        if *interaction == Interaction::Pressed {
            scramble.status = ScrambleStatus::Confirming;
            spawn_confirmation_dialog(&mut commands);
        }
    }
}

fn spawn_confirmation_dialog(commands: &mut Commands) {
    commands
        .spawn((
            ScrambleConfirmDialog,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            // High z-index to overlay everything
            ZIndex(100),
        ))
        .with_children(|overlay| {
            // Dialog box
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(24.0)),
                        row_gap: Val::Px(16.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.95)),
                ))
                .with_children(|dialog| {
                    // Message
                    dialog.spawn((
                        Text::new("Scramble the cube?\nThis will reset undo history."),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                        TextLayout::new_with_justify(JustifyText::Center),
                    ));

                    // Button row
                    dialog
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Yes button
                            row.spawn((
                                ScrambleConfirmYes,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor(Color::srgba(0.3, 0.8, 0.3, 0.6)),
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(Color::srgba(0.2, 0.5, 0.2, 0.8)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Yes"),
                                    TextFont { font_size: 14.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });

                            // No button
                            row.spawn((
                                ScrambleConfirmNo,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor(Color::srgba(0.8, 0.3, 0.3, 0.6)),
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.8)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("No"),
                                    TextFont { font_size: 14.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                });
        });
}

/// System: handle confirmation dialog interaction.
pub fn handle_scramble_confirmation(
    mut commands: Commands,
    mut scramble: ResMut<ScrambleQueue>,
    yes_query: Query<&Interaction, (Changed<Interaction>, With<ScrambleConfirmYes>)>,
    no_query: Query<&Interaction, (Changed<Interaction>, With<ScrambleConfirmNo>)>,
    dialog_query: Query<Entity, With<ScrambleConfirmDialog>>,
) {
    if scramble.status != ScrambleStatus::Confirming {
        return;
    }

    let confirmed = yes_query.iter().any(|i| *i == Interaction::Pressed);
    let cancelled = no_query.iter().any(|i| *i == Interaction::Pressed);

    if confirmed {
        scramble.moves = generate_scramble_moves();
        scramble.status = ScrambleStatus::Active;
        // Remove dialog
        for entity in &dialog_query {
            commands.entity(entity).despawn_recursive();
        }
    } else if cancelled {
        scramble.status = ScrambleStatus::Idle;
        for entity in &dialog_query {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System: process the next move from the scramble queue when animation is idle.
pub fn process_scramble_queue(
    mut commands: Commands,
    mut scramble: ResMut<ScrambleQueue>,
    mut animation: ResMut<FaceRotationAnimation>,
    cubies: Query<(Entity, &Cubie)>,
) {
    if scramble.status != ScrambleStatus::Active || animation.active {
        return;
    }

    let Some(mv) = scramble.moves.pop_front() else {
        return; // Queue empty — finish_scramble will handle it
    };

    let affected: Vec<Entity> = cubies
        .iter()
        .filter(|(_, c)| mv.axis.layer(c.grid_position) == mv.layer)
        .map(|(e, _)| e)
        .collect();

    if affected.is_empty() {
        return;
    }

    let pivot = commands
        .spawn((
            RotationPivot,
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    for &entity in &affected {
        commands.entity(entity).set_parent(pivot);
    }

    let angle = if mv.clockwise {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    };

    *animation = FaceRotationAnimation {
        active: true,
        pivot_entity: Some(pivot),
        affected_cubies: affected,
        rotation_axis: mv.axis.to_vec3(),
        target_angle: angle,
        current_angle: 0.0,
        duration: 0.1, // Fast animation for scramble
        elapsed: 0.0,
        move_data: mv,
        origin: ActionOrigin::Scramble,
    };
}

/// System: after the last scramble move completes, reset history and return to idle.
pub fn finish_scramble(
    mut scramble: ResMut<ScrambleQueue>,
    mut history: ResMut<ActionHistory>,
    animation: Res<FaceRotationAnimation>,
) {
    if scramble.status != ScrambleStatus::Active {
        return;
    }

    // Wait until both queue is empty and animation is done
    if !scramble.moves.is_empty() || animation.active {
        return;
    }

    // Clear history — scramble is a state reset
    history.undo_stack.clear();
    history.redo_stack.clear();

    scramble.status = ScrambleStatus::Idle;
}

/// System: disable scramble button during active scramble.
pub fn update_scramble_button(
    scramble: Res<ScrambleQueue>,
    animation: Res<FaceRotationAnimation>,
    mut query: Query<(&Children, &mut BackgroundColor), With<ScrambleButton>>,
    mut text_query: Query<&mut TextColor>,
) {
    let enabled = scramble.status == ScrambleStatus::Idle && !animation.active;

    let active_bg = Color::srgba(0.2, 0.2, 0.25, 0.8);
    let inactive_bg = Color::srgba(0.2, 0.2, 0.25, 0.4);
    let active_text = Color::srgba(1.0, 1.0, 1.0, 0.9);
    let inactive_text = Color::srgba(1.0, 1.0, 1.0, 0.4);

    for (children, mut bg) in &mut query {
        *bg = BackgroundColor(if enabled { active_bg } else { inactive_bg });
        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(if enabled { active_text } else { inactive_text });
            }
        }
    }
}

