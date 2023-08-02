use super::{Action, GameOverText, InGameOverMenu};
use bevy::prelude::*;

// All the possible messages to be shown when a game over occurs
const GAME_OVER_MESSAGES: GameOverText = GameOverText {
    messages: [
        "Better luck next time!",
        "Game Over! Try again?",
        "Wow that was bad.",
        "Out of all the possibilities,\nyou managed to execute\nthe single worst one.",
        "Maybe try lowering the\ndifficulty?",
    ],
};

// Create the Game Over menu
pub fn spawn_ui(mut commands: Commands, assets: Res<AssetServer>) {
    let font: Handle<Font> = assets.load("fonts/FiraSans-Bold.ttf");

    let button_style = Style {
        size: Size::new(Val::Px(175.0), Val::Px(50.0)),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: crate::ui::TEXT_COLOUR,
    };

    // Root element
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Px(40.0),
                        bottom: Val::Auto,
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            InGameOverMenu,
        ))
        .with_children(|parent| {
            // Game over message
            parent.spawn((
                GAME_OVER_MESSAGES,
                TextBundle::from_section(
                    GAME_OVER_MESSAGES.pick_random(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 46.0,
                        color: crate::ui::TEXT_COLOUR,
                    },
                )
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            ));
            // Sub-list for the buttons
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            margin: UiRect::top(Val::Px(150.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    },
                    InGameOverMenu,
                ))
                .with_children(|parent| {
                    // Retry button
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: crate::ui::BUTTON_BASE.into(),
                                ..default()
                            },
                            Action::Retry,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Retry", text_style.clone()));
                        });
                    // Back to main menu button
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: crate::ui::BUTTON_BASE.into(),
                                ..default()
                            },
                            Action::ToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Main menu", text_style.clone()));
                        });
                    // Quit game button
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: crate::ui::BUTTON_BASE.into(),
                                ..default()
                            },
                            Action::Exit,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Quit", text_style.clone()));
                        });
                });
        });
}
