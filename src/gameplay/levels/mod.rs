pub mod level1;
pub mod level2;
pub mod level3;
use std::time::Duration;

use crate::{gameplay::{bullet::Bullet, enemy::Boss, player::{EnemiesKilled, Player}, shared::Movement, collectables::{spawn_collectables, magnetise_all}, GameplayState}, GameState};

use super::{
    collisions::{self, ColliderType},
    loading::BackgroundHandle,
    shared::{physics::*, METRE},
};
use bevy::prelude::*;
use bevy::sprite::ColorMesh2dBundle;

// Level border
#[derive(Component)]
pub enum Wall {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, States)]
pub enum CurrentLevel {
    #[default]
    None,
    One,
    Two,
    Three,
    Endless,
}

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<CurrentLevel>()
            .add_systems(
                (
                    level1::spawn_boss,
                    reset_enemies_killed,
                )
                .in_schedule(OnEnter(CurrentLevel::One))
            )
            .add_systems(
                (
                    level1::enemy_movement,
                    level1::spawn_enemies,
                    level1::boss_movement,
                ).in_set(OnUpdate(CurrentLevel::One))
            )
            .add_systems(
                (
                    convert_leftover_bullets,
                ).in_schedule(OnExit(CurrentLevel::One))
            )
            .add_systems(
                (level2::spawn_boss, level2::setup_level, reset_enemies_killed, magnetise_all)
                .in_schedule(OnEnter(CurrentLevel::Two))
            )
            .add_systems(
                (
                    level2::spawn_enemies,
                    level2::enemy_movement,
                    level2::boss_movement,
                ).in_set(OnUpdate(CurrentLevel::Two))
            )
            .add_systems(
                (
                    convert_leftover_bullets,
                ).in_schedule(OnExit(CurrentLevel::Two))
            )
            .add_systems(
                (level3::spawn_boss, level3::setup_level, reset_enemies_killed, magnetise_all)
                .in_schedule(OnEnter(CurrentLevel::Three))
            )
            .add_systems(
                (
                    level3::spawn_enemies,
                    level3::enemy_movement,
                    level3::boss_movement,
                ).in_set(OnUpdate(CurrentLevel::Three))
            );
    }
}

pub fn check_won(bosses: Query<&Boss>, enemies_killed: Query<&EnemiesKilled, With<Player>>) -> bool {
    if bosses.iter().len() == 0 {
        for enemies_killed_instance in enemies_killed.iter() {
            if enemies_killed_instance.get_current_level() >= 15 {
                return true;
            }
        }
    }
    false
}

pub fn convert_leftover_bullets(bullets: Query<(Entity, &ColliderType, &Transform), With<Bullet>>, mut commands: Commands, asset_server: Res<AssetServer>) {
    for (bullet, kind, transform) in bullets.iter() {
        if *kind == ColliderType::EnemyBullet {
            if let Some(entity) = commands.get_entity(bullet) {
                entity.despawn_recursive();
            }
            spawn_collectables(&mut commands, 1, 0, transform, &asset_server, Movement::absolute(Vec2::new(0.0, -4.0), Vec2::ZERO));
        }
    }
}

pub fn setup_levels(mut commands: Commands, mut next_state: ResMut<NextState<CurrentLevel>>) {
    commands.insert_resource(SpawnEnemyTimer::default());
    next_state.set(CurrentLevel::One);
}

pub fn remove_level(mut next_state: ResMut<NextState<CurrentLevel>>) {
    next_state.set(CurrentLevel::None);
}

// Timer for spawning normal enemies
#[derive(Resource, Debug, Deref, DerefMut)]
pub struct SpawnEnemyTimer(pub Timer);
impl Default for SpawnEnemyTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1600), TimerMode::Once))
    }
}

// Level background image/texture, with a panning speed.
#[derive(Component)]
pub struct LevelBackground {
    pan_speed: f32,
}

pub fn setup_background(
    mut commands: Commands,
    bg_handle: Res<BackgroundHandle>,
    images: Res<Assets<Image>>,
    windows: Query<&Window>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Get the primary window of the game
    let window = windows.get_single().unwrap();
    // Get the size of the window (x, y)
    let (w_width, w_height) = (window.width(), window.height());
    // Get the size of the background image
    let bg_size = images.get(&bg_handle).unwrap().size();
    // Calculate the ratio between the image and window
    // so that the image can be scaled to fit the window.
    let scale_width = w_width / bg_size.x;
    let scale_height = w_height / bg_size.y;

    // The background here is split into two parts. This is so that there can be a seamless
    // vertical panning of the background image.
    // Part 1 of the background
    commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(bg_size).into()).into(),
            material: materials.add(ColorMaterial::from(bg_handle.0.clone())),
            transform: Transform::from_scale(Vec3::new(scale_width, scale_height, 1.0)),
            ..default()
        },
        LevelBackground { pan_speed: 100.0 },
    ));
    // Part 2 of the background
    commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(bg_size).into()).into(),
            material: materials.add(ColorMaterial::from(bg_handle.0.clone())),
            transform: Transform {
                // This part of the background is spawned at the top of the window
                translation: Vec3::new(0.0, w_height, 0.0),
                scale: Vec3::new(scale_width, scale_height, 1.0),
                ..default()
            },
            ..default()
        },
        LevelBackground { pan_speed: 100.0 },
    ));
}

pub fn pan_background(
    mut background: Query<(&LevelBackground, &mut Transform)>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    // First we get the coordinates of the bottom of the screen.
    // Then, every background part is translated down by pan_speed * dt.

    // As the midpoint of one part of the image reaches
    // the bottom of the screen, the midpoint of the other part reaches the middle of the screen
    // and the first part gets teleported to the top of the screen to continue the fake
    // panning effect.

    let bottom = -(windows.get_single().unwrap().height());
    for (background, mut transform) in background.iter_mut() {
        transform.translation.y -= time.delta_seconds() * background.pan_speed;
        // Quick fix for bug: Check 1 unit earlier to fix periodic seam.
        if transform.translation.y <= bottom + 1.0 {
            transform.translation.y = -bottom; // -bottom = top
        }
    }
}

pub fn create_playfield(mut commands: Commands, windows: Query<&Window>) {
    let window = windows.get_single().unwrap();

    // Get coordinates of window edges so that the walls can be spawned there.

    // vertical = top coordinates, -vertical = bottom coordinates
    let vertical = window.height() / 2.;
    // horizontal = right coordinates, -horizontal = left coordinates
    let horizontal = window.width() / 2.;

    // Arbitrary value for the relative "height" of a wall
    let cross_axis = 4.0 * METRE;

    for (wall, position, width, height) in [
        (
            Wall::Right,
            // The right wall needs to be placed at horizontal + cross_axis
            // to correct for its width of cross_axis
            Vec3::new(horizontal + cross_axis, 0.0, 0.0),
            // Width of the right wall
            cross_axis,
            // Height of the right wall, as tall as the game window
            vertical,
        ),
        (
            Wall::Left,
            // The left wall needs to be placed at -horizontal - cross_axis
            // to correct for its width of cross_axis
            Vec3::new(-horizontal - cross_axis, 0.0, 0.0),
            // Width of the left wall
            cross_axis,
            // Height of the left wall, as tall as the game window
            vertical,
        ),
        (
            Wall::Top,
            // The top wall needs to be placed at vertical + cross_axis
            // to correct for its height of cross_axis
            Vec3::new(0.0, vertical + cross_axis, 0.0),
            // Width of the top wall, as wide as the game window
            horizontal,
            // Height of the top wall
            cross_axis,
        ),
        (
            Wall::Bottom,
            // The bottom wall needs to be placed at -vertical - cross_axis
            // to correct for its height of cross_axis
            Vec3::new(0.0, -vertical - cross_axis / 1.2, 0.0),
            // Width of the bottom wall, as wide as the game window
            horizontal,
            // Height of the bottom wall
            cross_axis,
        ),
    ] {
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(position),
                // Make the level borders invisible during gameplay.
                visibility: Visibility::Hidden,
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(width, height),
            ColliderType::Wall,
            ColliderType::Wall.collision_group(),
            // SolverGroups required for automatic collisions from the physics engine
            SolverGroups::new(
                collisions::WALL_COL,
                collisions::PLAYER_COL | collisions::ENEMY_COL,
            ),
            ActiveEvents::COLLISION_EVENTS,
            wall,
        ));
    }
}

pub fn advance_level(current_level: Res<State<CurrentLevel>>, mut next_level: ResMut<NextState<CurrentLevel>>, mut next_gamestate: ResMut<NextState<GameState>>, mut next_gameplaystate: ResMut<NextState<GameplayState>>) {
    match current_level.0 {
        CurrentLevel::One => next_level.set(CurrentLevel::Two),
        CurrentLevel::Two => next_level.set(CurrentLevel::Three),
        CurrentLevel::Three => {
            next_level.set(CurrentLevel::None);
            next_gameplaystate.set(GameplayState::None);
            next_gamestate.set(GameState::GameWon);
        },
        CurrentLevel::None | CurrentLevel::Endless => {}
    }
}

fn reset_enemies_killed(mut enemies_killed: Query<&mut EnemiesKilled, With<Player>>) {
    for mut e in enemies_killed.iter_mut() {
        e.reset_current();
    }
}
