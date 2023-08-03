mod main_menu;
mod options;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::despawn_component;
use crate::GameState;

// Define menu states
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Hash, States)]
enum MenuState {
    MainMenu,
    Options,
    #[default]
    None,
}

// Enum of all button actions in the menu state
#[derive(Component)]
enum Action {
    StartGameplay,
    GoToOptions,
    GoToMenu,
    Exit,
    InvertFocus,
    Sound,
    VolumeUp,
    VolumeDown,
}

// Marker for UI objects that exist in the main menu
#[derive(Component)]
struct InMainMenu;

// Marker for UI objects that exist in the options menu
#[derive(Component)]
struct InOptionsMenu;

pub struct LandingScreenPlugin;

impl Plugin for LandingScreenPlugin {
    fn build(&self, app: &mut App) {
        // if cfg!(debug_assertions) {
        //     app.add_system(debug_menu_state);
        // }

        app.add_state::<MenuState>()
            .add_systems(OnEnter(GameState::Menu), setup)
            .add_systems(OnEnter(MenuState::MainMenu), main_menu::setup)
            .add_systems(OnExit(MenuState::MainMenu), despawn_component::<InMainMenu>)
            .add_systems(OnEnter(MenuState::Options), options::setup)
            .add_systems(Update, options::update_option_text.run_if(in_state(MenuState::Options)))
            .add_systems(OnExit(MenuState::Options), despawn_component::<InOptionsMenu>)
            .add_systems(Update, (crate::ui::colour_buttons, button_interactions).run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), despawn_component::<InMainMenu>);
    }
}

fn setup(mut next_state: ResMut<NextState<MenuState>>) {
    next_state.set(MenuState::MainMenu);
}

// Handle all possible button interactions in the menus
#[allow(clippy::type_complexity)]
fn button_interactions(
    interaction: Query<(&Interaction, &Action), (Changed<Interaction>, With<Button>)>,
    mut exit: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_options: ResMut<crate::GameOptions>,
) {
    for (interaction, action) in interaction.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                Action::StartGameplay => {
                    game_state.set(GameState::Gameplay);
                    menu_state.set(MenuState::None);
                }
                Action::InvertFocus => game_options.set_invert_focus(),
                Action::GoToOptions => menu_state.set(MenuState::Options),
                Action::GoToMenu => menu_state.set(MenuState::MainMenu),
                Action::Exit => exit.send(AppExit),
                Action::VolumeUp => {
                    let current_volume = game_options.get_volume();
                    game_options.set_volume(current_volume + 0.1);
                }
                Action::VolumeDown => {
                    let current_volume = game_options.get_volume();
                    game_options.set_volume(current_volume - 0.1);
                }
                _ => {}
            }
        }
    }
}

fn debug_menu_state(menu_state: Res<State<MenuState>>, timer: Res<crate::DebugTimer>) {
    if timer.finished() {
        info!("MenuState: {:?}", menu_state);
    }
}
