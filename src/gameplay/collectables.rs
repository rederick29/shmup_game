use crate::gameplay::{player::Player, shared::magnetise_to};
use super::collisions::ColliderType;
use super::shared::physics::*;
use super::shared::Movement;
use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectableType {
    Power,
    Score,
}

#[derive(Component)]
pub struct Collectable {
    pub kind: CollectableType,
}

// Lifetime for despawning and updating visuals of Collectables after
// some time has passed since the creation of a Collectable.
#[derive(Component, Debug)]
pub struct CollectableLifetime {
    // Lifetime timer --> How long should the Collectable be in the world for
    pub l_timer: Timer,
    // Visual timer --> General timer for updating any other data on a Collectable
    pub v_timer: Timer,
    // For tracking a boolean state of a Collectable. Intended use is with the v_timer
    flash: bool,
}
impl CollectableLifetime {
    pub const fn new(lifetime: Timer, flashing: Timer) -> Self {
        Self {
            l_timer: lifetime,
            v_timer: flashing,
            flash: false,
        }
    }

    pub fn get_flash(&self) -> bool {
        self.flash
    }

    // Change boolean state of `flash`
    pub fn invert_flash(&mut self) {
        self.flash = !self.flash;
    }
}

// Make the collectables flash every v_timer seconds if they have less than half of their lifetime
// remaining until being despawn. Each collectable flashes according to its own timer and state
// They are despawned once their lifetime is over
pub fn manage_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut collectables: Query<(Entity, &mut Sprite, &mut CollectableLifetime), With<Collectable>>,
) {
    for (entity, mut sprite, mut lifetime) in collectables.iter_mut() {
        // Increment the timers by the delta time between game updates.
        lifetime.v_timer.tick(time.delta());
        lifetime.l_timer.tick(time.delta());

        // Short circuit to the next iteration if more than half the lifetime is remaining
        if lifetime.l_timer.remaining() >= lifetime.l_timer.duration() / 2 {
            continue;
        }

        if lifetime.v_timer.finished() {
            // Update the collectable's sprite colour/tint based on its flash state
            // if the v_timer is finished. If flash is true, a darker tint is applied
            // and the sprite is made 40% transparent.
            // Otherwise, restore to normal opaque sprite.
            sprite.color = if lifetime.get_flash() {
                Color::rgba(0.8, 0.8, 0.8, 0.6)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            };
            lifetime.invert_flash();
        }

        if lifetime.l_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// Spawns n(_type) collectables around a point.
pub fn spawn_collectables(
    commands: &mut Commands,
    n_score: u8,
    n_power: u8,
    target: &Transform,
    assets: &AssetServer,
    movement: Movement,
) {
    for _ in 0..n_score {
        spawn_collectable_around(
            commands,
            target,
            assets,
            movement.clone(),
            CollectableType::Score,
        );
    }
    for _ in 0..n_power {
        spawn_collectable_around(
            commands,
            target,
            assets,
            movement.clone(),
            CollectableType::Power,
        );
    }
}

// Spawns a single Collectable of a certaint type around a target point with a given movement
pub fn spawn_collectable_around(
    commands: &mut Commands,
    target: &Transform,
    assets: &AssetServer,
    movement: Movement,
    kind: CollectableType,
) {
    // Choose the spawn point by randomly generating two real numbers, both floats between -20 and
    // 20 (inclusive). The numbers are added to the orginal target position's x and y components so
    // that the collectable is spawned some random (x, y) away from the target.
    let mut r_thread = rand::thread_rng();
    let r_transform = Transform {
        translation: Vec3 {
            x: target.translation.x + r_thread.gen_range(-20.0..=20.0),
            y: target.translation.y + r_thread.gen_range(-20.0..=20.0),
            // Furthest layer in the background is used as they could obstruct gameplay
            // elements such as the player sprite and hitbox otherwise.
            z: 0.0,
        },
        ..*target
    };

    // Spawn collectible with a 15-second lifetime and 2 Hz flashing update rate.
    // The sprite is chosen based on the kind of Collectable it is
    commands.spawn((
        Collectable { kind },
        CollectableLifetime::new(
            Timer::new(Duration::from_secs(15), TimerMode::Once),
            Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        ),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            texture: match kind {
                CollectableType::Score => assets.load("sprites/energy-pickup.png"),
                CollectableType::Power => assets.load("sprites/power-pickup.png"),
            },
            transform: r_transform,
            ..default()
        },
        Collider::ball(8.0),
        RigidBody::Dynamic,
        Velocity::zero(),
        ColliderType::Collectable,
        ColliderType::Collectable.collision_group(),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
        movement,
    ));
}

pub fn magnetise_to_player(
    mut collectables: Query<(&mut Movement, &Transform), With<Collectable>>,
    player_t: Query<&Transform, With<Player>>,
) {
    const STRENGTH: f32 = 15.0;
    const MINIMUM_DISTANCE: f32 = 30.0;

    let Ok(player_t) = player_t.get_single() else { return };

    for (mut movement, transform) in collectables.iter_mut() {
        if (player_t.translation - transform.translation).length() <= MINIMUM_DISTANCE {
            magnetise_to(&mut movement, transform, player_t, STRENGTH, false);
        }
    }
}

pub fn magnetise_all(
    mut collectables: Query<(&mut Movement, &Transform), With<Collectable>>,
    player_t: Query<&Transform, With<Player>>,
) {
    const STRENGTH: f32 = 40.0;

    let Ok(player_t) = player_t.get_single() else { return };

    for (mut movement, transform) in collectables.iter_mut() {
        magnetise_to(&mut movement, transform, player_t, STRENGTH, false);
    }
}
