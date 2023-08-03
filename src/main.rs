mod game_over;
mod gameplay;
mod landing_screen;
mod ui;
mod win_game;

use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
// use bevy_editor_pls::prelude::EditorPlugin;
#[cfg(not(target_family = "wasm"))]
use bevy_hanabi::HanabiPlugin;

const DEBUG_TIMER_DURATION: f32 = 5.0;

// Define all game states
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Paused,
    GameOver,
    Gameplay,
    GameWon,
}

#[derive(Clone, Copy, Debug, Default, Deref, DerefMut, PartialEq, Eq, Resource)]
pub struct HighScore(pub u64);

// Collection of global game options
#[derive(Clone, PartialEq, PartialOrd, Debug, Resource)]
pub struct GameOptions {
    volume: f32,
    invert_focus: bool,
}

impl GameOptions {
    // Volume is a value between 0 and 1
    pub fn set_volume(&mut self, volume: f32) {
        if volume >= 1. {
            self.volume = 1.;
        } else if volume <= 0. {
            self.volume = 0.;
        } else {
            self.volume = volume;
        }
    }
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    pub fn set_invert_focus(&mut self) {
        self.invert_focus = !self.invert_focus;
    }
    pub fn get_focus(&self) -> bool {
        self.invert_focus
    }
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            volume: 0.5,
            invert_focus: false,
        }
    }
}

fn main() {
    let mut app = App::new();
    // Check if running as debug
    if cfg!(debug_assertions) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!(
                    "{} - {} - debug",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ),
                resolution: (600., 800.).into(),
                resizable: true,
                mode: bevy::window::WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<DebugTimer>()
        .add_systems(Startup, debug_startup_game_state)
        .add_systems(Update, (tick_debug_timer, debug_game_state))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EntityCountDiagnosticsPlugin);
        //.add_plugin(EditorPlugin);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!(
                    "{} - {} - release",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ),
                resolution: (600., 800.).into(),
                resizable: false,
                mode: bevy::window::WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }));
    }


    app.add_systems(Startup, spawn_camera)
        // Particle effects creator and renderer
        .add_state::<GameState>()
        .init_resource::<GameOptions>()
        .init_resource::<HighScore>()
        .add_plugins(landing_screen::LandingScreenPlugin)
        .add_plugins(game_over::GameOverPlugin)
        .add_plugins(gameplay::GameplayPlugin)
        .add_plugins(win_game::WinGamePlugin);

    #[cfg(not(target_family = "wasm"))]
    app.add_plugins(HanabiPlugin);

    app.run();
}

#[derive(Resource, Deref, DerefMut)]
pub struct DebugTimer(Timer);
impl Default for DebugTimer {
    fn default() -> Self {
        DebugTimer(Timer::from_seconds(
            DEBUG_TIMER_DURATION,
            TimerMode::Repeating,
        ))
    }
}

fn tick_debug_timer(time: Res<Time>, mut timer: ResMut<DebugTimer>) {
    timer.tick(time.delta());
}

fn debug_game_state(game_state: Res<State<GameState>>, timer: Res<DebugTimer>) {
    if timer.finished() {
        info!("GameState: {:?}", game_state);
    }
}

fn debug_startup_game_state(game_state: Res<State<GameState>>) {
    info!("Initial GameState: {:?}", game_state);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// General despawn everything with a component.
fn despawn_component<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        if let Some(entity) = commands.get_entity(entity) {
            entity.despawn_recursive();
        }
    }
}
