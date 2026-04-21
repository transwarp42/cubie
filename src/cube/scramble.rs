use std::collections::VecDeque;

use bevy::prelude::*;
use rcuber::generator::Generator;
use rcuber::moves::Move as RcuberMove;
use rcuber::solver::min2phase::Min2PhaseSolver;

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

/// Marker for the reset button.
#[derive(Component)]
pub struct ResetButton;

/// Event fired when the cube should be reset to solved state.
#[derive(Event)]
pub struct ResetCubeEvent;

/// Marker for the confirmation dialog overlay.
#[derive(Component)]
pub struct ScrambleConfirmDialog;

/// Marker for the confirm (yes) button.
#[derive(Component)]
pub struct ScrambleConfirmYes;

/// Marker for the cancel (no) button.
#[derive(Component)]
pub struct ScrambleConfirmNo;

/// Convert an rcuber Move to one or more CubeMoves.
/// For 180° moves (X2), two identical 90° moves are returned.
/// For opposite-side faces (D, L, B), the clockwise direction is inverted
/// because "clockwise" in our model means CW when viewed from the positive axis.
pub(super) fn rcuber_move_to_cube_moves(m: RcuberMove) -> Vec<CubeMove> {
    match m {
        RcuberMove::U  => vec![CubeMove { axis: RotationAxis::Y, layer:  1, clockwise: true }],
        RcuberMove::U3 => vec![CubeMove { axis: RotationAxis::Y, layer:  1, clockwise: false }],
        RcuberMove::U2 => vec![CubeMove { axis: RotationAxis::Y, layer:  1, clockwise: true },
                               CubeMove { axis: RotationAxis::Y, layer:  1, clockwise: true }],
        RcuberMove::D  => vec![CubeMove { axis: RotationAxis::Y, layer: -1, clockwise: false }],
        RcuberMove::D3 => vec![CubeMove { axis: RotationAxis::Y, layer: -1, clockwise: true }],
        RcuberMove::D2 => vec![CubeMove { axis: RotationAxis::Y, layer: -1, clockwise: true },
                               CubeMove { axis: RotationAxis::Y, layer: -1, clockwise: true }],
        RcuberMove::R  => vec![CubeMove { axis: RotationAxis::X, layer:  1, clockwise: true }],
        RcuberMove::R3 => vec![CubeMove { axis: RotationAxis::X, layer:  1, clockwise: false }],
        RcuberMove::R2 => vec![CubeMove { axis: RotationAxis::X, layer:  1, clockwise: true },
                               CubeMove { axis: RotationAxis::X, layer:  1, clockwise: true }],
        RcuberMove::L  => vec![CubeMove { axis: RotationAxis::X, layer: -1, clockwise: false }],
        RcuberMove::L3 => vec![CubeMove { axis: RotationAxis::X, layer: -1, clockwise: true }],
        RcuberMove::L2 => vec![CubeMove { axis: RotationAxis::X, layer: -1, clockwise: true },
                               CubeMove { axis: RotationAxis::X, layer: -1, clockwise: true }],
        RcuberMove::F  => vec![CubeMove { axis: RotationAxis::Z, layer:  1, clockwise: true }],
        RcuberMove::F3 => vec![CubeMove { axis: RotationAxis::Z, layer:  1, clockwise: false }],
        RcuberMove::F2 => vec![CubeMove { axis: RotationAxis::Z, layer:  1, clockwise: true },
                               CubeMove { axis: RotationAxis::Z, layer:  1, clockwise: true }],
        RcuberMove::B  => vec![CubeMove { axis: RotationAxis::Z, layer: -1, clockwise: false }],
        RcuberMove::B3 => vec![CubeMove { axis: RotationAxis::Z, layer: -1, clockwise: true }],
        RcuberMove::B2 => vec![CubeMove { axis: RotationAxis::Z, layer: -1, clockwise: true },
                               CubeMove { axis: RotationAxis::Z, layer: -1, clockwise: true }],
        _ => vec![], // Ignore wide/slice/rotation moves (shouldn't appear in min2phase solutions)
    }
}

/// Generate a random-state scramble using the Kociemba two-phase algorithm.
///
/// 1. Generate a random valid cube state
/// 2. Solve it with min2phase to get: random_state → solved
/// 3. Invert the solution to get: solved → random_state (the scramble)
fn generate_scramble_moves() -> VecDeque<CubeMove> {
    let random_cube = Generator::random();
    let mut solver = Min2PhaseSolver { cube: random_cube };
    let solution = solver.solve();

    // Invert the solution: reverse order and invert each move
    let inverted: Vec<RcuberMove> = solution
        .moves
        .iter()
        .rev()
        .map(|m| m.get_inverse())
        .collect();

    inverted
        .into_iter()
        .flat_map(rcuber_move_to_cube_moves)
        .collect()
}

/// Spawn the scramble button (left of Labels, right of Undo/Redo).
pub fn spawn_scramble_button(mut commands: Commands) {
    // Reset button (right of Scramble)
    commands
        .spawn((
            ResetButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(399.0),
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
                Text::new("Reset"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ));
        });

    // Scramble button
    commands
        .spawn((
            ScrambleButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(309.0),
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
    solve: Res<super::solver::SolveQueue>,
    query: Query<&Interaction, (Changed<Interaction>, With<ScrambleButton>)>,
) {
    if scramble.status != ScrambleStatus::Idle
        || solve.status != super::solver::SolveStatus::Idle
        || animation.active
    {
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
    solve: Res<super::solver::SolveQueue>,
    mut scramble_query: Query<(&Children, &mut BackgroundColor), With<ScrambleButton>>,
    mut reset_query: Query<(&Children, &mut BackgroundColor), (With<ResetButton>, Without<ScrambleButton>)>,
    mut text_query: Query<&mut TextColor>,
) {
    let enabled = scramble.status == ScrambleStatus::Idle
        && solve.status == super::solver::SolveStatus::Idle
        && !animation.active;

    let active_bg = Color::srgba(0.2, 0.2, 0.25, 0.8);
    let inactive_bg = Color::srgba(0.2, 0.2, 0.25, 0.4);
    let active_text = Color::srgba(1.0, 1.0, 1.0, 0.9);
    let inactive_text = Color::srgba(1.0, 1.0, 1.0, 0.4);

    for (children, mut bg) in scramble_query.iter_mut().chain(reset_query.iter_mut()) {
        *bg = BackgroundColor(if enabled { active_bg } else { inactive_bg });
        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(if enabled { active_text } else { inactive_text });
            }
        }
    }
}

/// System: handle reset button click → fire ResetCubeEvent.
pub fn handle_reset_input(
    mut events: EventWriter<ResetCubeEvent>,
    scramble: Res<ScrambleQueue>,
    animation: Res<FaceRotationAnimation>,
    solve: Res<super::solver::SolveQueue>,
    query: Query<&Interaction, (Changed<Interaction>, With<ResetButton>)>,
) {
    if scramble.status != ScrambleStatus::Idle
        || solve.status != super::solver::SolveStatus::Idle
        || animation.active
    {
        return;
    }

    for interaction in &query {
        if *interaction == Interaction::Pressed {
            events.send(ResetCubeEvent);
        }
    }
}

/// System: execute cube reset — despawn all cubies, reset state, respawn.
pub fn execute_reset(
    mut commands: Commands,
    mut events: EventReader<ResetCubeEvent>,
    mut cube_state: ResMut<CubeState>,
    mut history: ResMut<ActionHistory>,
    cubies: Query<Entity, With<Cubie>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if events.read().next().is_none() {
        return;
    }
    // Consume remaining events
    events.read().for_each(drop);

    // Despawn all existing cubies and their children
    for entity in &cubies {
        commands.entity(entity).despawn_recursive();
    }

    // Reset logical state
    *cube_state = CubeState::solved();

    // Clear history
    history.undo_stack.clear();
    history.redo_stack.clear();

    // Respawn cube (reuse spawn_cube logic)
    super::spawn::spawn_cube(commands, meshes, materials, cube_state.into());
}

