use bevy::prelude::*;

use super::animation::{ActionOrigin, FaceRotationAnimation};
use super::model::*;
use super::rotation::RotationPivot;
use super::scramble::ScrambleQueue;

/// Resource storing the undo/redo action stacks.
#[derive(Resource, Default)]
pub struct ActionHistory {
    pub undo_stack: Vec<CubeMove>,
    pub redo_stack: Vec<CubeMove>,
}

impl ActionHistory {
    /// Record a new regular action: push to undo, clear redo.
    pub fn push_action(&mut self, mv: CubeMove) {
        self.undo_stack.push(mv);
        self.redo_stack.clear();
    }

    /// Pop from undo stack and push to redo stack. Returns the move to reverse.
    pub fn undo(&mut self) -> Option<CubeMove> {
        let mv = self.undo_stack.pop()?;
        self.redo_stack.push(mv);
        Some(mv)
    }

    /// Pop from redo stack and push to undo stack. Returns the move to replay.
    pub fn redo(&mut self) -> Option<CubeMove> {
        let mv = self.redo_stack.pop()?;
        self.undo_stack.push(mv);
        Some(mv)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Marker for the undo button.
#[derive(Component)]
pub struct UndoButton;

/// Marker for the redo button.
#[derive(Component)]
pub struct RedoButton;

/// Spawn undo/redo buttons in the top-right corner.
pub fn spawn_undo_redo_buttons(mut commands: Commands) {
    // Undo button
    commands
        .spawn((
            UndoButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(130.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.4)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Undo"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
            ));
        });

    // Redo button
    commands
        .spawn((
            RedoButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(188.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.4)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Redo"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
            ));
        });
}


/// System: handle undo/redo button clicks and keyboard shortcuts.
/// When triggered, starts a rotation animation via the same pipeline.
pub fn handle_undo_redo_input(
    mut commands: Commands,
    mut history: ResMut<ActionHistory>,
    mut animation: ResMut<FaceRotationAnimation>,
    keyboard: Res<ButtonInput<KeyCode>>,
    undo_query: Query<&Interaction, (Changed<Interaction>, With<UndoButton>)>,
    redo_query: Query<&Interaction, (Changed<Interaction>, With<RedoButton>)>,
    cubies: Query<(Entity, &Cubie)>,
    scramble: Res<ScrambleQueue>,
) {
    if animation.active || scramble.is_active() {
        return;
    }

    // Determine if undo or redo was requested
    let ctrl = keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    let undo_requested = undo_query.iter().any(|i| *i == Interaction::Pressed)
        || (ctrl && keyboard.just_pressed(KeyCode::KeyZ)
            && !keyboard.pressed(KeyCode::ShiftLeft) && !keyboard.pressed(KeyCode::ShiftRight));

    let redo_requested = redo_query.iter().any(|i| *i == Interaction::Pressed)
        || (ctrl && keyboard.just_pressed(KeyCode::KeyZ)
            && (keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)))
        || (ctrl && keyboard.just_pressed(KeyCode::KeyY));

    let (mv, origin) = if undo_requested {
        if let Some(original) = history.undo() {
            (original.inverse(), ActionOrigin::Undo)
        } else {
            return;
        }
    } else if redo_requested {
        if let Some(original) = history.redo() {
            (original, ActionOrigin::Redo)
        } else {
            return;
        }
    } else {
        return;
    };

    // Start rotation animation (same logic as start_face_rotation but bypassing DragState)
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
        duration: 0.2,
        elapsed: 0.0,
        move_data: mv,
        origin,
    };
}

/// System: update button visuals based on stack state.
pub fn update_undo_redo_buttons(
    history: Res<ActionHistory>,
    mut undo_query: Query<(&Children, &mut BackgroundColor), With<UndoButton>>,
    mut redo_query: Query<(&Children, &mut BackgroundColor), (With<RedoButton>, Without<UndoButton>)>,
    mut text_query: Query<&mut TextColor>,
) {
    let active_bg = Color::srgba(0.2, 0.2, 0.25, 0.8);
    let inactive_bg = Color::srgba(0.2, 0.2, 0.25, 0.4);
    let active_text = Color::srgba(1.0, 1.0, 1.0, 0.9);
    let inactive_text = Color::srgba(1.0, 1.0, 1.0, 0.4);

    for (children, mut bg) in &mut undo_query {
        let enabled = history.can_undo();
        *bg = BackgroundColor(if enabled { active_bg } else { inactive_bg });
        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(if enabled { active_text } else { inactive_text });
            }
        }
    }

    for (children, mut bg) in &mut redo_query {
        let enabled = history.can_redo();
        *bg = BackgroundColor(if enabled { active_bg } else { inactive_bg });
        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(if enabled { active_text } else { inactive_text });
            }
        }
    }
}

