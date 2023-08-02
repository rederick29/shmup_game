use crate::gameplay::{
    bullet::AttackPattern,
    bullet::Bullet,
    bullet::BulletGroup,
    collisions::ColliderType,
    enemy,
    enemy::{Attacks, Boss, Enemy},
    loading::Atlases,
    shared::Formation,
    shared::Movement,
    shared::Name,
    shared::MetaSpriteAtlas,
    GameplayTime,
    levels::SpawnEnemyTimer,
};
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[allow(unused)]
fn setup_level() {
    // Change background
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnEnemyTimer>,
    atlases: Res<Atlases<'static>>,
) {
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }
    let attacks = Attacks::new(
        vec![AttackPattern {
            bullet_group: BulletGroup {
                collider_type: ColliderType::EnemyBullet,
                number: 15,
                formation: Formation::circular(false, 20.0),
                bullet: Bullet::new(5.0, 5.0),
                ..default()
            },
            movement: Movement::new(
                Vec2::ZERO,
                Vec2::ZERO,
                true,
                Vec2::new(0.0, 3.0),
                Vec2::new(0.0, 2.0),
            ),
            cd: Timer::from_seconds(0.8, TimerMode::Once),
            icd: None,
            current_bullet: 0,
        }],
        Timer::new(Duration::from_secs(10), TimerMode::Once),
    );

    let spawn_point = Transform {
        translation: Vec3::new(rand::thread_rng().gen_range(-150..150) as f32, 300.0, 0.2),
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
                    formation: Formation::harmonic(false, 50.0, 20.0, 2.0),
                    number: 30,
                    collider_type: ColliderType::EnemyBullet,
                    bullet: Bullet::new(5.0, 20.0),
                    ..default()
                },
                Movement::new(
                    Vec2::ZERO,
                    Vec2::ZERO,
                    true,
                    Vec2::ZERO,
                    Vec2::new(0.0, 6.0),
                ),
                Timer::new(Duration::from_millis(3000), TimerMode::Once),
                Some(Timer::new(Duration::from_millis(100), TimerMode::Once)),
            ),
            AttackPattern::new(
                BulletGroup {
                    formation: Formation::circular(false, 20.0),
                    number: 30,
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
                Timer::new(Duration::from_millis(3000), TimerMode::Once),
                Some(Timer::new(Duration::from_millis(100), TimerMode::Once)),
            ),
            AttackPattern::new(
                BulletGroup {
                    formation: Formation::linear(Transform::default(), Vec2::ZERO),
                    number: 5,
                    collider_type: ColliderType::EnemyBullet,
                    bullet: Bullet::new(5.0, 20.0),
                    ..default()
                },
                Movement::new(
                    Vec2::ZERO,
                    Vec2::ZERO,
                    true,
                    Vec2::new(0.0, 15.0),
                    Vec2::ZERO,
                ),
                Timer::new(Duration::from_millis(1200), TimerMode::Once),
                Some(Timer::new(Duration::from_millis(60), TimerMode::Once)),
            ),
        ],
        Timer::new(Duration::from_secs(10), TimerMode::Once),
    );

    let spawn_point = Transform {
        translation: Vec3::new(100.0, 100.0, 0.0),
        ..default()
    };

    let sprite = MetaSpriteAtlas {
        sprite: TextureAtlasSprite {
            color: Color::rgb(0.1, 0.6, 0.3),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        texture_atlas: Some(atlases.get("sprites/enemy-medium.png").expect("Couldn't get enemy texture atlas.").clone()),
        collider: Collider::cuboid(25.0, 25.0),
        ..default()
    };

    enemy::spawn_boss(
        &mut commands,
        Name::from("Big Boss"),
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
    let amplitude = 2.0;
    let frequency = 0.5;
    let mut phase_difference = 10.0;

    for mut mov in enemies.iter_mut() {
        phase_difference += 4.0;
        mov.v_local.x =
            amplitude * (TAU * frequency * time.elapsed_secs() + phase_difference).sin();
    }
}

pub fn boss_movement(mut boss: Query<&mut Movement, With<Boss>>, time: Res<GameplayTime>) {
    let Ok(mut movement) = boss.get_single_mut() else { return; };
    use std::f32::consts::TAU;
    let amplitude = 5.0;
    let frequency = 0.1;
    let phase_difference = 10.0;
    movement.v_local = Vec2::new(amplitude * (TAU * frequency * time.elapsed_secs() + phase_difference).sin(), 0.0);
}
