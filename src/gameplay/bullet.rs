use super::{
    collisions::ColliderType,
    shared::{physics::*, ExtraSpriteInfo, Formation, Movement},
};
use bevy::prelude::*;
use std::time::Duration;

// Marker component. This is what makes an entity a bullet
#[derive(Component, Clone, Copy, Debug)]
pub struct Bullet {
    damage: f32,
    max_damage: f32,
}

impl Bullet {
    pub fn new(damage: f32, max_damage: f32) -> Self {
        Self {
            damage: if damage > max_damage {
                warn!("Damage set higher than maximum on creation. Maximum will take precedence and cap it.");
                max_damage
            } else {
                damage
            },
            max_damage,
        }
    }

    pub fn set_damage(&mut self, damage: f32) {
        if damage > self.max_damage {
            warn!("Higher damage set than max damage. Capping!");
            self.damage = self.max_damage;
        } else {
            self.damage = damage;
        }
    }

    pub fn get_damage(&self) -> f32 {
        self.damage
    }

    pub fn set_max(&mut self, max_damage: f32) {
        if max_damage < self.damage {
            warn!("Maximum damage set to less than current damage. Capping!");
            self.damage = max_damage;
        }
        self.max_damage = max_damage;
    }

    pub fn get_max(&self) -> f32 {
        self.max_damage
    }

    pub fn set(&mut self, damage: f32, max_damage: f32) {
        self.set_max(max_damage);
        self.set_damage(damage);
    }
}

// Struct for defining a set of bullets that are similar and have a formation
#[derive(Debug, Clone)]
pub struct BulletGroup {
    pub collider_type: ColliderType,
    pub number: u16,
    pub origin: Transform,
    pub formation: Formation,
    pub bullet: Bullet,
}

// A "default" BulletGroup value consists of one bullet at world spawn with default
// sprite and formation.
impl Default for BulletGroup {
    fn default() -> Self {
        Self {
            collider_type: ColliderType::None,
            number: 1,
            origin: Transform::default(),
            formation: Formation::default(),
            bullet: Bullet::new(1.0, 1.0),
        }
    }
}

impl BulletGroup {
    // Procedure for spawning a single bullet of a given bullet group into the world.
    // The bullet to spawn out of the group is given as `i`
    pub fn spawn_single<T: ExtraSpriteInfo>(
        &self,
        commands: &mut Commands,
        movement: Movement,
        i: u16,
        sprite: T,
    ) {
        let spawn_point = self.formation.transform(i, self.number, self.origin);

        let mut binding = commands.spawn((
            sprite.bundle(spawn_point),
            self.bullet,
            RigidBody::Dynamic,
            Velocity::zero(),
            movement,
            sprite.collider(),
            self.collider_type,
            self.collider_type.collision_group(),
            Sensor,
        ));
        if let Some(collider) = sprite.grazing_collider() {
            binding.with_children(|parent| {
                parent.spawn((collider, ColliderType::Graze, ColliderType::Graze.collision_group(), Sensor));
            });
        }
    }
    pub fn spawn_all<T: ExtraSpriteInfo + Clone>(
        &self,
        commands: &mut Commands,
        movement: Movement,
        sprite: T,
    ) {
        for i in 0..self.number {
            self.spawn_single(commands, movement.clone(), i, sprite.clone());
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct AttackPattern {
    pub bullet_group: BulletGroup,
    // This movement is to be applied to the Bullet Group
    pub movement: Movement,
    // CD is a cooldown timer for how long the game should wait between multiple attack groups
    pub cd: Timer,
    // ICD is an internal cooldown timer for how long the game should wait between spawing
    // individual bullets of the Bullet Group.
    pub icd: Option<Timer>,
    // For using with the ICD as an iterator
    pub current_bullet: u16,
}

impl AttackPattern {
    pub const fn new(
        bullet_group: BulletGroup,
        movement: Movement,
        cooldown: Timer,
        internal_cooldown: Option<Timer>,
    ) -> Self {
        Self {
            bullet_group,
            movement,
            cd: cooldown,
            icd: internal_cooldown,
            current_bullet: 0,
        }
    }
}

impl Default for AttackPattern {
    fn default() -> Self {
        Self {
            bullet_group: BulletGroup::default(),
            movement: Movement::default(),
            cd: Timer::new(Duration::from_millis(10000), TimerMode::Once),
            icd: Some(Timer::new(Duration::from_millis(100), TimerMode::Once)),
            current_bullet: 0,
        }
    }
}
