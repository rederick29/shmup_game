mod bullet;
mod collectables;
mod collisions;
mod enemy;
mod event;
mod levels;
mod loading;
// Public for access in the game won screen
pub mod player;
pub mod shared;
mod ui;

use crate::{despawn_component, gameplay::player::Player};
use crate::GameState;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{RapierConfiguration, RapierDebugRenderPlugin},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, States)]
pub enum GameplayState {
    Loading,
    Playing,
    #[default]
    None,
}

// Custom execution stages for handling collisions and updating player data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum CustomSet {
    Collisions,
    UpdateStats,
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugins(RapierDebugRenderPlugin::default());
        }

        app.add_state::<GameplayState>()
            .add_event::<event::TakeDamageEvent>()
            .add_event::<event::DespawnEvent>()
            .add_event::<event::GameOverEvent>();

        app
            .insert_resource::<loading::Atlases>(Default::default())
            .insert_resource::<loading::BackgroundHandle>(Default::default())
            .insert_resource::<collisions::Collisions>(collisions::Collisions::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
                shared::METRE,
            ))
            .add_plugins(levels::LevelsPlugin)
            // Enter Gameplay
            .add_systems(OnEnter(GameState::Gameplay), setup)
            // Begin Loading / Early Load
            .add_systems(OnEnter(GameplayState::Loading),
                (
                    loading::load_background,
                    loading::load_texture_atlases,
                    ui::create_stats_list,
                )
            )
            // Early loading finished, switch to GameplayState::Playing
            .add_systems(Update,
                loading::finish_loading
                    .run_if(
                        loading::check_background_loaded, //.and_then(loading::check_particles_loaded)
                                                          //.and_then(loading::check_atlases_loaded)
                    )
                    .run_if(in_state(GameplayState::Loading))
            )
            // OnEnter
            .add_systems(OnEnter(GameplayState::Playing),
                (
                    setup_gameplay,
                    levels::setup_background,
                    levels::create_playfield,
                    levels::setup_levels,
                    player::spawn_player,
                )
            )
            // OnUpdate
            .add_systems(Update,
                (
                    back_to_menu,
                    tick_gameplay,
                    collisions::handle_collisions,
                    collectables::manage_lifetimes,
                    collectables::magnetise_to_player,
                    player::spawn_player_bullet,
                    // when the player hits X on the keyboard,
                    // `uses_special` is true, therefore `special_attack` runs
                    // when `special_attack` changes `Specials`, `used_special` is true
                    // so `magnetise_all` runs. If magnetise_all runs after
                    // `special_attack`, it does not do what is expected due to events
                    // generated in `special_attack` being processed at a later stage.
                    // This ordering causes `used_special` to pick up changes from
                    // `special_attack` one frame late, giving time for events
                    // to be processed.
                    collectables::magnetise_all.run_if(player::used_special),
                    player::special_attack.run_if(player::uses_special).after(collectables::magnetise_all),
                    player::move_player,
                    enemy::enemy_attack,
                )
                    .run_if(in_state(GameplayState::Playing)),
            )
            .add_systems(Update,
                (
                    shared::move_object::<bullet::Bullet>,
                    shared::move_object::<enemy::Enemy>,
                    shared::move_object::<collectables::Collectable>,
                    levels::pan_background,
                    levels::advance_level.run_if(levels::check_won),
                ).run_if(in_state(GameplayState::Playing))
            )
            // OnExit -- Despawn all game objects
            .add_systems(OnExit(GameplayState::Playing),
                (
                    remove_player,
                    despawn_component::<bullet::Bullet>,
                    despawn_component::<enemy::Enemy>,
                    despawn_component::<levels::Wall>,
                    despawn_component::<ui::GameplayUI>,
                    despawn_component::<levels::LevelBackground>,
                    despawn_component::<collectables::Collectable>,
                    levels::remove_level,
                )
            )
            // Configure custom sets
            // Collisions update stage is after the normal Update stage
            .configure_set(PostUpdate,
                CustomSet::Collisions
                    .run_if(in_state(GameplayState::Playing)),
            )
            // UpdateStats stage is after the Collision stage
            .configure_set(Update,
                CustomSet::UpdateStats
                    .after(CustomSet::Collisions)
                    .run_if(in_state(GameplayState::Playing)),
            )
            // Collisions
            .add_systems(Update,
                (
                    collisions::handle_bullet_col,
                    collisions::handle_player_col,
                    collisions::handle_enemy_col,
                    collisions::handle_collectable_col,
                )
                .in_set(CustomSet::Collisions)
            )
            // UpdateStats
            .add_systems(Update,
                (
                    event::take_damage,
                    event::score_on_enemy_damage,
                    event::despawn_entity,
                    event::create_collectables_on_despawn,
                    event::game_over,
                    ui::update_health_bar::<enemy::BossHealthBar, enemy::Boss>,
                    ui::update_health_bar::<player::PlayerHealthBar, player::Player>,
                    ui::update_counter_ui::<player::ScoreText>,
                    ui::update_counter_ui::<player::GrazeText>,
                    ui::update_counter_ui::<player::PowerText>,
                    ui::update_counter_ui::<player::SpecialsText>,
                    ui::update_counter_ui::<player::EnemiesKilledText>,
                    collisions::cleanup_collisions,
                )
                .in_set(CustomSet::UpdateStats)
            );

        #[cfg(not(target_family = "wasm"))]
        app
            .insert_resource::<loading::ParticleEffects>(Default::default())
            .add_systems(OnEnter(GameplayState::Loading), loading::load_particle_effects)
            .add_systems(OnExit(GameplayState::Playing), despawn_component::<player::PlayerBooster>);

    }
}

fn setup(mut next_state: ResMut<NextState<GameplayState>>) {
    // Start loading the game when entering GameState::Gameplay
    next_state.set(GameplayState::Loading);
}

// A resource to keep track of time since started playing the game
// (excluding time spent in menus and resets when retry-ing)
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct GameplayTime(Stopwatch);

fn setup_gameplay(mut commands: Commands, mut physics: ResMut<RapierConfiguration>) {
    // No automatic gravity required from the physics simulation
    physics.gravity = Vec2::ZERO;

    // Insert any resources needed for the Playing state.
    commands.insert_resource(GameplayTime::default());
    commands.insert_resource::<collisions::Collisions>(collisions::Collisions::default());
    commands.insert_resource(player::PlayerAttackCD::default());
}

// Update the GameplayTime timer
fn tick_gameplay(mut g_time: ResMut<GameplayTime>, r_time: Res<Time>) {
    g_time.tick(r_time.delta());
}

// Intercept the Escape key on the keyvboard to return the player back to the main menu
fn back_to_menu(
    input: Res<Input<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut gameplay_state: ResMut<NextState<GameplayState>>,
) {
    if input.pressed(KeyCode::Escape) {
        game_state.set(GameState::Menu);
        gameplay_state.set(GameplayState::None);
    }
}

fn remove_player(
    mut commands: Commands,
    mut visibility: Query<(Entity, &mut Visibility), With<Player>>,
    next_state: Res<NextState<GameState>>,
) {
    if next_state.0 == Some(GameState::GameWon) {
        let (_, mut visibility) = visibility.get_single_mut().expect("Couldn't hide player.");
        *visibility = Visibility::Hidden;
    } else {
        let Ok((entity, _)) = visibility.get_single() else { return; };
        if let Some(entity) = commands.get_entity(entity) {
            entity.despawn_recursive();
        }
    }
}
