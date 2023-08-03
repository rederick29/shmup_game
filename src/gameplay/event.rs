use super::{
    collectables::spawn_collectables,
    collisions::ColliderType,
    player::{Player, Score},
    shared::{Counter, Health, Movement},
    ui::Link,
    GameplayState,
};
use crate::{GameState, gameplay::player::EnemiesKilled};
use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct TakeDamageEvent {
    entity: Entity,
    entity_type: Option<ColliderType>,
    damage: f32,
}
impl TakeDamageEvent {
    pub fn new(entity: Entity, entity_type: Option<ColliderType>, damage: f32) -> Self {
        Self {
            entity,
            entity_type,
            damage,
        }
    }
}

pub fn take_damage(
    mut damage_ev: EventReader<TakeDamageEvent>,
    mut game_over_ev: EventWriter<GameOverEvent>,
    mut health: Query<(&mut Health, Option<&Link>)>,
    mut enemies_killed: Query<&mut EnemiesKilled, With<Player>>,
    mut despawn_ev: EventWriter<DespawnEvent>,
) {
    for event in damage_ev.iter() {
        let Ok((mut hp, health_bar)) = health.get_mut(event.entity) else { continue; };
        // Update the affected entity by taking away the damage value from its health component.
        if hp.current > event.damage {
            hp.current -= event.damage;
        } else {
            // If the damage is >= health, then this event would kill the entity, so we despawn the
            // entity and its health bar UI element if it exists.
            // Furthermore, if the receiving entity is a Player, this results in a Game Over event.

            if let Some(health_bar) = health_bar {
                despawn_ev.send(DespawnEvent::new(health_bar.0, true));
            }
            if let Some(entity_type) = event.entity_type {
                if entity_type == ColliderType::Player {
                    game_over_ev.send(GameOverEvent);
                }
                else if entity_type == ColliderType::Enemy {
                    enemies_killed.iter_mut().for_each(|mut k| k.increment());
                }
            }
            despawn_ev.send(
                DespawnEvent::new(event.entity, false)
                    .with_score(5)
                    .with_power(3),
            );
        }
    }
}

pub fn score_on_enemy_damage(
    mut damage_ev: EventReader<TakeDamageEvent>,
    mut player_score: Query<&mut Score, With<Player>>,
) {
    for event in damage_ev.iter() {
        if let Some(entity_type) = event.entity_type {
            if entity_type == ColliderType::Enemy {
                player_score.iter_mut().for_each(|mut s| s.add(20));
            }
        }
    }
}

#[derive(Event)]
pub struct DespawnEvent {
    entity: Entity,
    recursive: bool,
    drop_score: u8,
    drop_power: u8,
}
impl DespawnEvent {
    pub fn new(entity: Entity, recursive: bool) -> Self {
        Self {
            entity,
            recursive,
            drop_score: 0,
            drop_power: 0,
        }
    }

    pub fn with_score(mut self, collectibles: u8) -> Self {
        self.drop_score = collectibles;
        self
    }

    pub fn with_power(mut self, collectibles: u8) -> Self {
        self.drop_power = collectibles;
        self
    }
}

pub fn despawn_entity(mut despawn_ev: EventReader<DespawnEvent>, mut commands: Commands) {
    for event in despawn_ev.iter() {
        let Some(mut entity_commands) = commands.get_entity(event.entity) else { continue };
        // Recursive despawning removes the entity as well as its children
        // from the world.
        match event.recursive {
            true => entity_commands.despawn_recursive(),
            false => entity_commands.despawn(),
        }
    }
}

// Spawns collectibles when despawning an entity when the Despawn Event has 'drop_score'
// or 'drop_power' set to some number other than 0.
pub fn create_collectables_on_despawn(
    mut commands: Commands,
    mut despawn_ev: EventReader<DespawnEvent>,
    transforms: Query<&Transform>,
    assets: Res<AssetServer>,
) {
    for event in despawn_ev.iter() {
        let Ok(target) = transforms.get(event.entity) else { continue; };
        let movement = Movement::new(
            Vec2::new(0.0, -7.0),
            Vec2::ZERO,
            false,
            Vec2::ZERO,
            Vec2::ZERO,
        );
        spawn_collectables(
            &mut commands,
            event.drop_score,
            event.drop_power,
            target,
            &assets,
            movement,
        );
    }
}

#[derive(Default, Event)]
pub struct GameOverEvent;

pub fn game_over(
    mut game_over_ev: EventReader<GameOverEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut gameplay_state: ResMut<NextState<GameplayState>>,
) {
    // Here .iter().next() is used as there may be a case where more than one GameOverEvent is
    // received due to how the systems are being scheduled. Only one event is needed to be handled,
    // so the rest are ignored.
    if game_over_ev.iter().next().is_some() {
        gameplay_state.set(GameplayState::None);
        game_state.set(GameState::GameOver);
    }
}
