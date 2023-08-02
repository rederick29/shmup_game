use crate::gameplay::{
    bullet::AttackPattern,
    bullet::Bullet,
    bullet::BulletGroup,
    collisions::ColliderType,
    enemy,
    enemy::{Attacks, Boss, Enemy},
    loading::{Atlases, BackgroundHandle},
    shared::Formation,
    shared::Movement,
    shared::Name,
    shared::MetaSpriteAtlas,
    GameplayTime,
    levels::{SpawnEnemyTimer, LevelBackground},
};
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_rapier2d::prelude::*;
use rand::Rng;

pub fn setup_level(
    asset_server: Res<AssetServer>,
    mut background_handle: ResMut<BackgroundHandle>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut current_backgrounds: Query<&mut Handle<ColorMaterial>, With<LevelBackground>>,
) {
    let bg = asset_server.load("backgrounds/level_2.png");
    background_handle.0 = bg;
    for mut background in current_backgrounds.iter_mut() {
        *background = materials.add(ColorMaterial::from(background_handle.0.clone()));
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnEnemyTimer>,
    atlases: Res<Atlases<'static>>,
) {
    if timer.duration() != Duration::from_millis(1100) {
        timer.set_duration(Duration::from_millis(1100));
    }
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }
    let attacks = Attacks::new(
        vec![AttackPattern {
            bullet_group: BulletGroup {
                collider_type: ColliderType::EnemyBullet,
                number: 10,
                formation: Formation::circular(false, 10.0),
                bullet: Bullet::new(5.0, 5.0),
                ..default()
            },
            movement: Movement::relative(
                Vec2::new(0.0, 5.0),
                Vec2::new(0.0, -2.5),
            ),
            cd: Timer::from_seconds(0.8, TimerMode::Once),
            icd: None,
            current_bullet: 0,
        }],
        Timer::new(Duration::from_secs(10), TimerMode::Once),
    );

    let spawn_point = Transform {
        translation: Vec3::new(rand::thread_rng().gen_range(-250..250) as f32, 330.0, 0.2),
        ..default()
    };
    let sprite = MetaSpriteAtlas {
        sprite: TextureAtlasSprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        texture_atlas: Some(
            atlases
                .get("sprites/enemy-small.png")
                .expect("Couldn't get enemy texture atlas.")
                .clone(),
        ),
        collider: Collider::cuboid(10.0, 10.0),
        ..default()
    };

    timer.reset();
    enemy::spawn_enemy(&mut commands, spawn_point, attacks, sprite);
}

pub fn spawn_boss(mut commands: Commands, asset_server: Res<AssetServer>, atlases: Res<Atlases<'static>>) {
    let attacks = Attacks::new(
        vec![
            AttackPattern::new(
                BulletGroup {
                    formation: Formation::circular(false, 10.0),
                    number: 45,
                    collider_type: ColliderType::EnemyBullet,
                    bullet: Bullet::new(5.0, 20.0),
                    ..default()
                },
                Movement::new(
                    Vec2::ZERO,
                    Vec2::ZERO,
                    true,
                    Vec2::new(0.0, 12.5),
                    Vec2::new(0.0, -4.0),
                ),
                Timer::new(Duration::from_millis(360), TimerMode::Once),
                Some(Timer::new(Duration::from_millis(8), TimerMode::Once)),
            ),
            AttackPattern::new(
                BulletGroup {
                    formation: Formation::harmonic(false, 20.0, 30.0, 4.0),
                    number: 35,
                    collider_type: ColliderType::EnemyBullet,
                    bullet: Bullet::new(5.0, 20.0),
                    ..default()
                },
                Movement::new(
                    Vec2::ZERO,
                    Vec2::ZERO,
                    true,
                    Vec2::new(0.0, 10.0),
                    Vec2::ZERO,
                ),
                Timer::new(Duration::from_millis(6000), TimerMode::Once),
                Some(Timer::new(Duration::from_millis(100), TimerMode::Once)),
            ),
        ],
        Timer::new(Duration::from_secs(10), TimerMode::Once),
    );

    let spawn_point = Transform {
        translation: Vec3::new(0.0, 80.0, 0.0),
        ..default()
    };

    let sprite = MetaSpriteAtlas {
        sprite: TextureAtlasSprite {
            color: Color::rgb(0.8, 0.1, 0.3),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        texture_atlas: Some(atlases.get("sprites/enemy-medium.png").expect("Couldn't get enemy texture atlas.").clone()),
        collider: Collider::cuboid(25.0, 25.0),
        ..default()
    };

    enemy::spawn_boss(
        &mut commands,
        Name::from("Bigger Boss"),
        spawn_point,
        attacks,
        asset_server,
        sprite,
    );
}

pub fn enemy_movement(
    time: Res<GameplayTime>,
    mut enemies: Query<&mut Movement, (With<Enemy>, Without<Boss>)>,
) {
    use std::f32::consts::TAU;
    let amplitude = 4.0;
    let frequency = 0.3;
    let mut phase_difference = 22.0;

    for mut mov in enemies.iter_mut() {
        phase_difference += 4.0;
        mov.v_local.x =
            amplitude * (TAU * frequency * time.elapsed_secs() + phase_difference).sin();
    }
}

pub fn boss_movement(mut boss: Query<(&mut Velocity, &mut Movement), With<Boss>>) {
    let Ok((mut velocity, mut movement)) = boss.get_single_mut() else { return; };
    movement.v_local = Vec2::new(0.0, 5.0);
    velocity.angvel = 2.0;
}

