mod menu;

use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use rand::Rng;

// Button actions enum
#[derive(Component)]
enum Action {
    Retry,
    ToMainMenu,
    Exit,
}

// Marker of UI items that exist in the Game Over screen
#[derive(Component)]
struct InGameOverMenu;

// Define all the game over texts (string array wrapper)
#[derive(Component)]
struct GameOverText {
    pub messages: [&'static str; 5],
}
impl GameOverText {
    pub fn pick_random(&self) -> &str {
        self.messages[rand::thread_rng().gen_range(0..self.messages.len())]
    }
}

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(menu::spawn_ui.in_schedule(OnEnter(GameState::GameOver)))
            .add_systems(
                (
                    button_interactions,
                    crate::ui::animate_text::<GameOverText>,
                    crate::ui::colour_buttons,
                )
                    .in_set(OnUpdate(GameState::GameOver)),
            )
            .add_system(
                crate::despawn_component::<InGameOverMenu>.in_schedule(OnExit(GameState::GameOver)),
            );
    }
}

// Handle all the button interactions in the game over screen
#[allow(clippy::type_complexity)]
fn button_interactions(
    interaction: Query<(&Interaction, &Action), (Changed<Interaction>, With<Button>)>,
    mut exit: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action) in interaction.iter() {
        if *interaction == Interaction::Clicked {
            match action {
                Action::Retry => game_state.set(GameState::Gameplay),
                Action::ToMainMenu => game_state.set(GameState::Menu),
                Action::Exit => exit.send(AppExit),
            }
        }
    }
}
