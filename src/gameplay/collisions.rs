use crate::gameplay::player::Graze;
use super::{
    bullet::Bullet,
    collectables::{Collectable, CollectableType},
    enemy::Enemy,
    event::{DespawnEvent, TakeDamageEvent},
    levels::Wall,
    player::{Player, Power, Score},
    shared::{physics::*, Counter, Movement},
};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_rapier2d::rapier::geometry::CollisionEventFlags;
use rand::Rng;

// Define all Collision Groups and Collision Filters so that
// all game objects interact as intended.
pub const PLAYER_COL: Group = Group::GROUP_1;
pub const WALL_COL: Group = Group::GROUP_2;
pub const ENEMY_COL: Group = Group::GROUP_3;
pub const ENEMY_BULLET_COL: Group = Group::GROUP_4;
pub const PLAYER_BULLET_COL: Group = Group::GROUP_5;
pub const COLLECTABLE_COL: Group = Group::GROUP_6;
pub const GRAZE_COL: Group = Group::GROUP_7;

pub const PLAYER_FILTER: Group = ENEMY_COL
    .union(ENEMY_BULLET_COL)
    .union(WALL_COL)
    .union(COLLECTABLE_COL)
    .union(GRAZE_COL);
pub const ENEMY_FILTER: Group = PLAYER_COL.union(PLAYER_BULLET_COL).union(WALL_COL);
pub const WALL_FILTER: Group = ENEMY_COL
    .union(PLAYER_COL)
    .union(PLAYER_BULLET_COL)
    .union(ENEMY_BULLET_COL)
    .union(COLLECTABLE_COL);
pub const PLAYER_BULLET_FILTER: Group = ENEMY_COL.union(WALL_COL);
pub const ENEMY_BULLET_FILTER: Group = PLAYER_COL.union(WALL_COL);
pub const COLLECTABLE_FILTER: Group = PLAYER_COL.union(WALL_COL);
pub const GRAZE_FILTER: Group = PLAYER_COL;

// Used for filtering collision handling by object type.
#[derive(Clone, Component, Copy, PartialEq, Eq, Debug)]
pub enum ColliderType {
    Player,
    PlayerBullet,
    Enemy,
    EnemyBullet,
    Wall,
    Collectable,
    Graze,
    None,
}
impl ColliderType {
    // Function for automatically generating collision groups for each collider type.
    pub fn collision_group(&self) -> CollisionGroups {
        use ColliderType::*;
        match self {
            Player => CollisionGroups::new(PLAYER_COL, PLAYER_FILTER),
            PlayerBullet => CollisionGroups::new(PLAYER_BULLET_COL, PLAYER_BULLET_FILTER),
            Enemy => CollisionGroups::new(ENEMY_COL, ENEMY_FILTER),
            EnemyBullet => CollisionGroups::new(ENEMY_BULLET_COL, ENEMY_BULLET_FILTER),
            Wall => CollisionGroups::new(WALL_COL, WALL_FILTER),
            Collectable => CollisionGroups::new(COLLECTABLE_COL, COLLECTABLE_FILTER),
            Graze => CollisionGroups::new(GRAZE_COL, GRAZE_FILTER),
            _ => {
                warn!("default collision group on ColliderType reached.");
                CollisionGroups::new(Group::NONE, Group::NONE)
            }
        }
    }
}

// Marks entitites that have been involved in a collision during the latest game update.
#[derive(Component)]
pub struct CollisionMarker;

// Data to be generated along with a collision. Includes information about the other entity that
// one entity has collided with.
#[derive(Component, Debug)]
pub struct CollisionData {
    pub other_type: ColliderType,
    pub other_entity: Entity,
    pub flags: CollisionEventFlags,
    pub started: bool,
}

// Memory for storing collision data for each entity involved in a collision.
// This is a hash table using the Entity IDs as keys and vectors (growable arrays) of
// CollisionData as values. This way it can be checked if an entity has collided by checking
// if it exists in the table. By storing the CollisionData in a vector, Entities that have
// collided multiple times in a single frame can be accounted for.
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct Collisions(HashMap<Entity, Vec<CollisionData>>);

// Receives collision events from the physics simulation of the game and
// populates the Collisions memory with the correct CollisionData.
pub fn handle_collisions(
    mut commands: Commands,
    colliders: Query<&ColliderType>,
    mut collisions: ResMut<Collisions>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for event in collision_events.iter() {
        let (entity1, entity2, flags, started) = match event {
            CollisionEvent::Started(e1, e2, flags) => (*e1, *e2, *flags, true),
            CollisionEvent::Stopped(e1, e2, flags) => (*e1, *e2, *flags, false),
        };

        // Retrieve the ColliderType of the entities that have collided.
        // If the entities do not have a ColliderType component they are ignored
        let Ok(entity1_type) = colliders.get(entity1) else { continue; };
        let Ok(entity2_type) = colliders.get(entity2) else { continue; };

        // Create collision data for entity 1
        let entity1_data = CollisionData {
            other_type: *entity2_type,
            other_entity: entity2,
            flags,
            started,
        };

        // Create collision data for entity 2
        let entity2_data = CollisionData {
            other_type: *entity1_type,
            other_entity: entity1,
            flags,
            started,
        };

        // Mark the two entities as having collided
        commands.entity(entity1).insert(CollisionMarker);
        commands.entity(entity2).insert(CollisionMarker);

        // Add the data into the table. It is first attempted to retrieve
        // the entity from the table in the case that it might have already collided
        // during the last frame. If no vector is returned then we create a new entry in
        // the table for the entity and add its collision data into a new vector.
        if let Some(vec) = collisions.get_mut(&entity1) {
            vec.push(entity1_data);
        } else {
            collisions.insert(entity1, vec![entity1_data]);
        }

        if let Some(vec) = collisions.get_mut(&entity2) {
            vec.push(entity2_data);
        } else {
            collisions.insert(entity2, vec![entity2_data]);
        }
    }
}

// Handles collisions for Bullet entities.
#[allow(clippy::type_complexity)]
pub fn handle_bullet_col(
    collisions: Res<Collisions>,
    mut despawn_ev: EventWriter<DespawnEvent>,
    mut damage_ev: EventWriter<TakeDamageEvent>,
    player_power: Query<&Power, With<Player>>,
    bullets: Query<(Entity, &ColliderType, &Bullet), With<CollisionMarker>>,
) {
    for (entity, bullet_type, bullet) in bullets.iter() {
        let Some(collisions) = collisions.get(&entity) else { continue; };
        for collision in collisions {
            let damage_dealt = if *bullet_type == ColliderType::PlayerBullet && collision.started {
                // Create variables needed for mapping the values
                let Ok(power) = player_power.get_single() else { continue; };
                let upper_damage = bullet.get_max();
                let lower_damage = bullet.get_damage();
                let upper_power = power.max() as f32;
                let input = power.get() as f32;

                // This is mapping a range, a_1..a_2, to another range, b_1..b_2
                // for an input value i. The formula is the following:
                // scaled_value = b_1 + ((i - a_1) * (b_2 - b_1) / (a_2 - a_1))
                // Since my a_1 is 0, I have simplified into the following.
                lower_damage + input * (upper_damage - lower_damage) / upper_power
            } else {
                bullet.get_damage()
            };
            match collision.other_type {
                ColliderType::Player => {
                    if !collision.started {
                        continue;
                    }
                    damage_ev.send(TakeDamageEvent::new(
                        collision.other_entity,
                        Some(collision.other_type),
                        damage_dealt,
                    ));
                    despawn_ev.send(DespawnEvent::new(entity, true));
                }
                ColliderType::Enemy => {
                    if !collision.started {
                        continue;
                    }
                    damage_ev.send(TakeDamageEvent::new(
                        collision.other_entity,
                        Some(collision.other_type),
                        damage_dealt,
                    ));
                    despawn_ev.send(DespawnEvent::new(entity, true));
                }
                ColliderType::Wall => {
                    if collision.started {
                        continue;
                    }
                    despawn_ev.send(DespawnEvent::new(entity, true));
                }
                _ => continue,
            }
        }
    }
}

// Handle collisions for Collectable entities.
pub fn handle_collectable_col(
    collisions: Res<Collisions>,
    mut despawn_ev: EventWriter<DespawnEvent>,
    mut collectables: Query<(Entity, &mut Movement, &Collectable), With<CollisionMarker>>,
    mut player_score: Query<&mut Score, With<Player>>,
    mut player_power: Query<&mut Power, With<Player>>,
    walls: Query<&Wall>,
) {
    for (entity, mut movement, collectable) in collectables.iter_mut() {
        let Some(collisions) = collisions.get(&entity) else { continue; };
        for collision in collisions {
            // Only act on the beginning of a collision event, ignoring the end.
            if collision.started {
                // If the collectable has been picked up by the player, make appropriate changes
                if collision.other_type == ColliderType::Player {
                    match collectable.kind {
                        CollectableType::Score => {
                            player_score.iter_mut().for_each(|mut s| s.add(50))
                        }
                        CollectableType::Power => {
                            player_power.iter_mut().for_each(|mut p| p.add(1))
                        }
                    }
                    // Despawn the entity
                    despawn_ev.send(DespawnEvent::new(entity, false));
                // If the collectable has collided with a level border, simulate simple bounces.
                } else if collision.other_type == ColliderType::Wall {
                    let mut r_thread = rand::thread_rng();
                    // Retrieves which wall the collectable collided with.
                    // Theoretically, this should never fail however that case is still handled by
                    // crashing the app with a message.
                    let wall = walls
                        .get(collision.other_entity)
                        .expect("Collided with a wall that doesn't exist!");
                    let random_value = r_thread.gen_range(-0.5..0.5);

                    match wall {
                        // For the left and right level borders, decrease the horizontal velocity
                        // of the collectable by (half +/- a random value from 0 to 0.5).
                        // The vertical velocity is also changed by a random number between -0.5
                        // and 0.5 (inclusive) to simulate different angles of incidnce/reflection
                        // as if it were bouncing off a rough surface.
                        Wall::Left | Wall::Right => {
                            movement.velocity.x = -movement.velocity.x / (2. + random_value);
                            movement.velocity.y += random_value;
                        }
                        // The same behaviour is simulated for a vertical collision but with the
                        // changes to x component before being applied to the y component now and
                        // vice-versa.
                        Wall::Top | Wall::Bottom => {
                            movement.velocity.y = -movement.velocity.y / (2.5 + random_value);
                            movement.velocity.x += random_value;
                        }
                    }
                }
            }
        }
    }
}

// Handles player collisions
pub fn handle_player_col(
    collisions: Res<Collisions>,
    mut damage_ev: EventWriter<TakeDamageEvent>,
    player: Query<Entity, (With<Player>, With<CollisionMarker>)>,
    mut graze: Query<&mut Graze, With<Player>>,
    mut score: Query<&mut Score, With<Player>>,
) {
    // There is only one player in the game so we can get_single()
    let Ok(player) = player.get_single() else { return; };
    let Some(collisions) = collisions.get(&player) else { return; };
    for collision in collisions {
        // The player takes 10 health points damage when collising with any enemy
        if collision.other_type == ColliderType::Enemy && collision.started {
            damage_ev.send(TakeDamageEvent::new(
                player,
                Some(ColliderType::Player),
                10.0,
            ));
        }
        if collision.other_type == ColliderType::Graze && collision.started {
            graze.iter_mut().for_each(|mut g| g.add(1));
            score.iter_mut().for_each(|mut s| s.increase_multiplier_by(0.01))
        }
    }
}

// Handles enemy collisions
pub fn handle_enemy_col(
    collisions: Res<Collisions>,
    mut damage_ev: EventWriter<TakeDamageEvent>,
    mut despawn_ev: EventWriter<DespawnEvent>,
    enemies: Query<Entity, (With<Enemy>, With<CollisionMarker>)>,
) {
    for enemy in enemies.into_iter() {
        let Some(collisions) = collisions.get(&enemy) else { continue; };
        for collision in collisions {
            if collision.other_type == ColliderType::Player && collision.started {
                // Enemy should also take damage by collisiding with the player
                damage_ev.send(TakeDamageEvent::new(enemy, None, 15.0));
            }
            if collision.flags.contains(CollisionEventFlags::SENSOR)
                && collision.other_type == ColliderType::Wall
                && !collision.started
            {
                despawn_ev.send(DespawnEvent::new(enemy, false));
            }
        }
    }
}

// Removes the CollisionMarker from enitites and removes collisions from the Collisions table.
// This is done after all the other collision related functions
pub fn cleanup_collisions(
    mut commands: Commands,
    mut collisions: ResMut<Collisions>,
    collided: Query<Entity, With<CollisionMarker>>,
) {
    for entity in collided.into_iter() {
        collisions.remove(&entity);
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.remove::<CollisionMarker>();
        }
    }
}
