use super::{
    bullet::AttackPattern,
    collisions::ColliderType,
    loading::Atlases,
    player::Player,
    shared::{
        physics::*, ExtraSpriteInfo, Formation, FormationShape, Health, MetaSpriteAtlas, Movement,
        Name, METRE, METRE_SQUARED,
    },
    ui::{create_health_bar, ObjectType},
};
use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Boss;

// Health Bar UI element for Boss-type enemies
#[derive(Component)]
pub struct BossHealthBar;
impl super::ui::ProgressBar for BossHealthBar {}

// Hold the possible atacks that one entity can choose from
#[derive(Component, Debug)]
pub struct Attacks {
    // Actual attack patterns
    attacks: Vec<AttackPattern>,
    // Current index in vec
    current_attack: usize,
    // Timer for switching between attacks
    switch_timer: Timer,
}

impl Attacks {
    pub const fn new(attacks: Vec<AttackPattern>, switch_timer: Timer) -> Self {
        Self {
            attacks,
            current_attack: 0,
            switch_timer,
        }
    }
    pub fn get_attacks(&self) -> Vec<AttackPattern> {
        self.attacks.clone()
    }

    pub fn get_attacks_ref(&self) -> &Vec<AttackPattern> {
        &self.attacks
    }

    pub fn get_attacks_mut(&mut self) -> &mut Vec<AttackPattern> {
        &mut self.attacks
    }

    pub fn get_current_attack(&self) -> usize {
        self.current_attack
    }

    pub fn get_current_attack_mut(&mut self) -> &mut usize {
        &mut self.current_attack
    }

    pub fn get_all_mut(&mut self) -> (&mut Vec<AttackPattern>, &mut usize, &mut Timer) {
        (
            &mut self.attacks,
            &mut self.current_attack,
            &mut self.switch_timer,
        )
    }
}

pub fn spawn_boss<T: ExtraSpriteInfo>(
    commands: &mut Commands,
    name: Name<'static>,
    spawn_point: Transform,
    attacks: Attacks,
    asset_server: Res<AssetServer>,
    sprite: T,
) {
    let health_bar = create_health_bar::<BossHealthBar>(
        commands,
        &asset_server,
        name.clone(),
        ObjectType::Enemy,
        BossHealthBar,
    );

    commands.spawn((
        sprite.bundle(spawn_point),
        sprite.collider(),
        attacks,
        name,
        Enemy,
        Boss,
        Health::new(300.0, None),
        RigidBody::Dynamic,
        ColliderType::Enemy,
        ColliderType::Enemy.collision_group(),
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        Movement::ZERO,
        super::ui::Link(health_bar),
    ));
}

pub fn enemy_attack(
    mut commands: Commands,
    mut enemy: Query<(&Transform, &mut Attacks), With<Enemy>>,
    player_t: Query<&Transform, With<Player>>,
    dt: Res<Time>,
    atlases: Res<Atlases<'static>>,
    state: Res<State<GameState>>,
) {
    for (transform, mut attacks) in enemy.iter_mut() {
        // Get number of attacks that the enemy can cycle through
        let attacks_number = attacks.attacks.len();

        // Retrieve the current attack
        let (attacks, current_attack_number, switch_timer) = attacks.get_all_mut();
        let mut attack = &mut attacks[*current_attack_number];

        // Whenever the game end or starts, reset the variables and timers
        if state.is_changed() {
            attack.cd.reset();
            if let Some(icd) = &mut attack.icd {
                icd.reset();
            };
            switch_timer.reset();
            *current_attack_number = 0;
            attack.current_bullet = 0;
        }

        // Tick attack timers.
        attack.cd.tick(dt.delta());

        // If the current bullet number is equal to or has gone over the total
        // number of bullets in the bullet_group, check if the attack cooldown is finished
        // so that current_bullet can be reset to 0 and the attack cooldown can be reset.
        if attack.current_bullet >= attack.bullet_group.number {
            if attack.cd.finished() {
                attack.current_bullet = 0u16;
                attack.cd.reset();
            }
            continue;
        }

        if let Some(icd) = &mut attack.icd {
            icd.tick(dt.delta());
        };
        switch_timer.tick(dt.delta());

        // Cycle through attacks by increasing current_attack by one until
        // the last is reached, after which the current_attack is reset back to 0
        if switch_timer.finished() {
            if *current_attack_number < attacks_number - 1 {
                *current_attack_number += 1;
            } else {
                *current_attack_number = 0;
            }
            switch_timer.reset();
        }

        // If there is an ICD and it is not yet finished, do not continue running
        // the function and return early.
        let timer = match &attack.icd {
            Some(icd) => icd,
            None => &attack.cd,
        };
        if !timer.finished() {
            continue;
        }

        // Actually spawn the attacks
        // Load bullet sprite
        let bullet_texture = atlases
            .get("sprites/enemy-projectile.png")
            .expect("Texture atlas not found!")
            .clone();

        // Create Meta Sprite for the bullet entity
        let meta_sprite = MetaSpriteAtlas {
            sprite: TextureAtlasSprite {
                custom_size: Some(METRE_SQUARED * 2.0),
                ..default()
            },
            texture_atlas: Some(bullet_texture),
            collider: Collider::ball(METRE / 2.5),
            // Value of 1.3 found through experimentation and what looks ok to me
            grazing_collider: Some(Collider::ball(METRE / 1.3)),
        };

        // Set the bullet_group origin transform to the enemy's position
        attack.bullet_group.origin = *transform;

        // If an attack is of Linear Formation, then it means that it was not
        // fully initialised when declared as it would require a target transform,
        // which can only be retrieved at runtime, when spawning the bullet, so
        // update the bullet_group formation so that the target transform is initialised
        // correctly.
        if attack.bullet_group.formation.kind == FormationShape::Linear {
            let player_transform = *player_t.get_single().unwrap_or(&Transform::default());
            attack.bullet_group.formation =
                Formation::linear(player_transform, meta_sprite.sprite.custom_size.unwrap());
        }

        // If there is an ICD in the attack pattern, create a custom loop that runs accross frames
        // by using current_bullet as an iterator, and manually increment it every time ICD finishes.
        if let Some(icd) = &mut attack.icd {
            attack.bullet_group.spawn_single(
                &mut commands,
                attack.movement.clone(),
                attack.current_bullet,
                meta_sprite,
            );

            if icd.finished() {
                attack.current_bullet += 1;
                icd.reset();
            }
        } else {
            // When there is no ICD, spawn all the bullets in the group at once, using just the CD
            // for timing attacks.
            attack
                .bullet_group
                .spawn_all(&mut commands, attack.movement.clone(), meta_sprite);
            attack.cd.reset();
        }
    }
}

// Spawns normal enemies
pub fn spawn_enemy<T: ExtraSpriteInfo>(
    commands: &mut Commands,
    spawn_point: Transform,
    attacks: Attacks,
    sprite: T,
) {
    commands.spawn((
        sprite.bundle(spawn_point),
        sprite.collider(),
        attacks,
        Enemy,
        Health::new(20.0, Some(20.0)),
        RigidBody::Dynamic,
        ColliderType::Enemy,
        ColliderType::Enemy.collision_group(),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
        Velocity::zero(),
        Movement::relative(Vec2::ZERO, Vec2::new(0.0, -3.0)),
    ));
}
