use crate::{gameplay::shared::Counter, despawn_component, HighScore};
use crate::GameState;
use crate::gameplay::player::{
    Player,
    Score,
    Graze,
    Power,
    EnemiesKilled,
    Specials
};
use bevy::app::AppExit;
use bevy::prelude::*;

#[derive(Component)]
enum Action {
    ToMainMenu,
    Exit,
}

#[derive(Component)]
struct InWinGameMenu;

pub struct WinGamePlugin;

impl Plugin for WinGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameWon), spawn_ui)
            .add_systems(Update,
                (
                    button_interactions,
                    crate::ui::colour_buttons,
                ).run_if(in_state(GameState::GameWon))
            )
            .add_systems(OnExit(GameState::GameWon), (crate::despawn_component::<InWinGameMenu>, despawn_component::<Player>));
    }
}

#[allow(clippy::type_complexity)]
fn button_interactions(
    interaction: Query<(&Interaction, &Action), (Changed<Interaction>, With<Button>)>,
    mut exit: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action) in interaction.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                Action::ToMainMenu => game_state.set(GameState::Menu),
                Action::Exit => exit.send(AppExit),
            }
        }
    }
}

// Create the Game Over menu
pub fn spawn_ui(
    mut commands: Commands,
    assets: Res<AssetServer>,
    player_data: Query<(&Specials, &Power, &Score, &Graze, &EnemiesKilled), With<Player>>,
    mut highscore: ResMut<HighScore>,
) {
    let Ok((specials, power, score, graze, enemies_killed)) = player_data.get_single() else { return; };
    let font: Handle<Font> = assets.load("fonts/FiraSans-Bold.ttf");

    if score.get() > highscore.0 {
       highscore.0 = score.get();
    }

    let button_style = Style {
        width: Val::Px(175.0),
        height: Val::Px(50.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: crate::ui::TEXT_COLOUR,
    };

    let base_text_bundle = || TextBundle::from_section(
        "",
        TextStyle {
            font: font.clone(),
            font_size: 23.0,
            color: crate::ui::TEXT_COLOUR,
        })
        .with_text_alignment(TextAlignment::Left)
        .with_style(Style {
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        }
    );

    let formatted_strings = [
        format!("Score: {}", score.get()),
        format!("Highscore: {}", highscore.0),
        format!("Power: {}", power.get()),
        format!("Specials remaining: {}", specials.get()),
        format!("Graze acquired: {}", graze.get()),
        format!("Enemies Killed: {}", enemies_killed.get()),
    ];

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
            InWinGameMenu,
        ))
        .with_children(|parent| {
            // Game won message
            parent.spawn((
                TextBundle::from_section(
                    "Congratulations, You Won!",
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

            for string in formatted_strings {
                let mut bundle = base_text_bundle();
                bundle.text.sections[0].value = string;
                parent.spawn(bundle);
            }

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
                    InWinGameMenu,
                ))
                .with_children(|parent| {
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
                            parent.spawn(TextBundle::from_section("Main menu", button_text_style.clone()));
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
                            parent.spawn(TextBundle::from_section("Quit", button_text_style.clone()));
                        });
                });
        });
}
