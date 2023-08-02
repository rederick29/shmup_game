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
            .add_system(setup.in_schedule(OnEnter(GameState::Menu)))
            .add_system(main_menu::setup.in_schedule(OnEnter(MenuState::MainMenu)))
            .add_system(despawn_component::<InMainMenu>.in_schedule(OnExit(MenuState::MainMenu)))
            .add_system(options::setup.in_schedule(OnEnter(MenuState::Options)))
            .add_system(options::update_option_text.in_set(OnUpdate(MenuState::Options)))
            .add_system(despawn_component::<InOptionsMenu>.in_schedule(OnExit(MenuState::Options)))
            .add_systems(
                (crate::ui::colour_buttons, button_interactions).in_set(OnUpdate(GameState::Menu)),
            )
            .add_system(despawn_component::<InMainMenu>.in_schedule(OnExit(GameState::Menu)));
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
        if *interaction == Interaction::Clicked {
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
