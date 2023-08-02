use crate::gameplay::event::DespawnEvent;

use super::{
    bullet::{Bullet, BulletGroup},
    collisions::{ColliderType, PLAYER_BULLET_COL},
    loading::{Atlases, ParticleEffects},
    shared::{physics::*, Counter, Formation, Health, MetaSprite, Movement, METRE, METRE_SQUARED},
    ui::{
        create_counter, create_health_bar, Link, ObjectType, ProgressBar, StatsList, UpdatingText,
    },
};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy)]
pub struct Specials {
    remaining: u8
}
impl Specials {
    pub fn new(number: u8) -> Self {
        Self {
            remaining: number
        }
    }
}

impl Counter for Specials {
    type Data = u8;

    fn set(&mut self, remaining_specials: Self::Data) {
        self.remaining = remaining_specials;
    }

    fn get(&self) -> Self::Data {
        self.remaining
    }

    fn add(&mut self, number: Self::Data) {
        self.remaining += number;
    }

    fn subtract(&mut self, number: Self::Data) {
        self.remaining -= number;
    }
}

#[derive(Component)]
pub struct SpecialsText {
    entity: Entity,
}

impl UpdatingText for SpecialsText {
    type DataHolder = Specials;

    fn original(&self) -> String {
        String::from("Specials:")
    }

    fn section(&self) -> usize {
        0
    }

    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Component, Debug, Default)]
pub struct Power {
    current: u16,
    max: u16,
}

impl Counter for Power {
    type Data = u16;

    fn set(&mut self, power: Self::Data) {
        // Cap the power to the maximum
        if power >= self.max {
            self.current = self.max;
        } else {
            self.current = power;
        }
    }

    fn get(&self) -> Self::Data {
        self.current
    }

    fn add(&mut self, power: Self::Data) {
        let current = self.get();
        self.set(current + power);
    }

    fn subtract(&mut self, power: Self::Data) {
        let current = self.get();
        self.set(if current <= power { 0 } else { current - power });
    }
}

impl Power {
    pub fn new(starting: u16, maximum: u16) -> Self {
        Self {
            current: if starting <= maximum {
                starting
            } else {
                warn!("Higher starting value than maximum provided to new Power instance.");
                0
            },
            max: maximum,
        }
    }

    pub fn max(&self) -> u16 {
        self.max
    }
}

#[derive(Component)]
pub struct PowerText {
    entity: Entity,
}

impl UpdatingText for PowerText {
    type DataHolder = Power;

    fn original(&self) -> String {
        String::from("Power:")
    }

    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Debug, Clone, Component)]
pub struct Score {
    score: u64,
    pre_multiplier_score: u64,
    multiplier: f32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            score: 0,
            pre_multiplier_score: 0,
            multiplier: 1.5,
        }
    }
}

impl Score {
    pub fn get_multiplier(&self) -> f32 {
        self.multiplier
    }

    pub fn increase_multiplier_by(&mut self, value: f32) {
        self.multiplier += value;
    }

    pub fn set_multiplier(&mut self, value: f32) {
        self.multiplier = value;
    }
}

impl Counter for Score {
    type Data = u64;

    // Bypasses multiplier
    fn set(&mut self, score: Self::Data) {
        self.score = score;
        self.pre_multiplier_score = score;
    }

    fn get(&self) -> Self::Data {
        self.score
    }

    fn add(&mut self, score: Self::Data) {
        self.pre_multiplier_score += score;
        self.score = (self.pre_multiplier_score as f32 * self.multiplier) as u64;
    }

    // Bypasses multiplier
    fn subtract(&mut self, score: Self::Data) {
        let current = self.get();
        self.set(if current <= score { 0 } else { current - score });
    }
}

#[derive(Component)]
pub struct ScoreText {
    entity: Entity,
}

impl UpdatingText for ScoreText {
    type DataHolder = Score;

    fn original(&self) -> String {
        String::from("Score:")
    }

    fn entity(&self) -> Entity {
        self.entity
    }

}

#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct Graze(u32);

impl Counter for Graze {
    type Data = u32;

    fn set(&mut self, value: Self::Data) {
        self.0 = value;
    }

    fn get(&self) -> Self::Data {
        self.0
    }

    fn add(&mut self, value: Self::Data) {
        self.set(self.get() + value);
    }

    fn subtract(&mut self, value: Self::Data) {
        self.set(self.get() - value);
    }
}

#[derive(Component)]
pub struct GrazeText {
    entity: Entity
}

impl UpdatingText for GrazeText {
    type DataHolder = Graze;

    fn original(&self) -> String {
        String::from("Graze:")
    }

    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Component)]
pub struct PlayerHealthBar;
impl ProgressBar for PlayerHealthBar {}

#[derive(Component, Debug, Default)]
pub struct EnemiesKilled {
    total: u16,
    current_level: u16,
}

impl Counter for EnemiesKilled {
    type Data = u16;

    fn set(&mut self, n: Self::Data) {
        self.total = n;
        self.current_level = n;
    }

    fn add(&mut self, n: Self::Data) {
        self.set(self.get() + n);
    }

    fn get(&self) -> Self::Data {
        self.total
    }

    fn subtract(&mut self, n: Self::Data) {
        self.set(self.get() - n);
    }
}

impl EnemiesKilled {
    pub fn new() -> EnemiesKilled {
        Self {
            total: 0,
            current_level: 0,
        }
    }

    pub fn get_current_level(&self) -> u16 {
        self.current_level
    }

    pub fn increment(&mut self) {
        self.total += 1;
        self.current_level += 1;
    }

    pub fn reset_current(&mut self) {
        self.current_level = 0;
    }
}

#[derive(Component)]
pub struct EnemiesKilledText {
    entity: Entity
}

impl UpdatingText for EnemiesKilledText {
    type DataHolder = EnemiesKilled;

    fn original(&self) -> String {
        String::from("Enemies Killed:")
    }

    fn entity(&self) -> Entity {
        self.entity
    }
}

// Cooldown between player attacks in order to have a set fire rate for the player
#[derive(Resource, Debug, Deref, DerefMut)]
pub struct PlayerAttackCD(Timer);
impl Default for PlayerAttackCD {
    fn default() -> Self {
        use std::time::Duration;
        Self(Timer::new(Duration::from_millis(200), TimerMode::Once))
    }
}

#[derive(Component)]
pub struct PlayerBooster;

pub fn spawn_player(
    mut commands: Commands,
    mut ui_list: Query<(Entity, &mut StatsList)>,
    atlases: Res<Atlases<'static>>,
    effects: Res<ParticleEffects<'static>>,
    assets: Res<AssetServer>,
) {
    let player_name = super::shared::Name::from("Player 1");
    let health_bar = PlayerHealthBar;
    let health_bar = create_health_bar::<PlayerHealthBar>(
        &mut commands,
        &assets,
        player_name.clone(),
        ObjectType::Player,
        health_bar,
    );

    let mut binding = commands
        .spawn((
            Player,
            Score::default(),
            Power::new(0, 500),
            Health::new(30.0, None),
            RigidBody::Dynamic,
            Velocity::zero(),
            Collider::ball(5.0),
            ColliderType::Player,
            ColliderType::Player.collision_group(),
            ActiveEvents::COLLISION_EVENTS,
            LockedAxes::ROTATION_LOCKED,
            Movement::new(
                Vec2::new(240.0, 240.0),
                Vec2::ZERO,
                false,
                Vec2::ZERO,
                Vec2::ZERO,
            ),
            player_name,
            Link(health_bar),
            SpriteSheetBundle {
                texture_atlas: atlases.get("sprites/white-plane3.png").unwrap().clone(),
                transform: Transform::from_translation(Vec3::new(0.0, -300.0, 0.1)),
                sprite: TextureAtlasSprite {
                    index: 5,
                    custom_size: Some(Vec2::new(48.0, 68.0)),
                    ..default()
                },
                ..default()
            },
        ));
    binding.insert((EnemiesKilled::default(), Specials::new(5), Graze::default()));

    let player_entity = binding
        .with_children(|parent| {
            parent.spawn((
                PlayerBooster,
                ParticleEffectBundle {
                    effect: ParticleEffect::new(effects.get("player_booster").unwrap().clone())
                        .with_z_layer_2d(Some(0.0)),
                    transform: Transform::from_translation(Vec3::new(0.0, -60.0, 0.0)),
                    ..default()
                },
            ));
        }).id();

    create_counter::<ScoreText>(
        &mut commands,
        &mut ui_list,
        &assets,
        ScoreText { entity: player_entity }
    );

    create_counter::<GrazeText>(
        &mut commands,
        &mut ui_list,
        &assets,
        GrazeText { entity: player_entity }
    );

    create_counter::<PowerText>(
        &mut commands,
        &mut ui_list,
        &assets,
        PowerText { entity: player_entity }
    );

    create_counter::<SpecialsText>(
        &mut commands,
        &mut ui_list,
        &assets,
        SpecialsText { entity: player_entity }
    );

    create_counter::<EnemiesKilledText>(
        &mut commands,
        &mut ui_list,
        &assets,
        EnemiesKilledText { entity: player_entity }
    );
}

pub fn uses_special(input: Res<Input<KeyCode>>) -> bool {
    input.just_pressed(KeyCode::X)
}

pub fn used_special(specials: Query<Ref<Specials>>) -> bool {
    for special in specials.iter() {
        if special.is_changed() {
            return true;
        }
    }
    false
}

pub fn special_attack(
    mut commands: Commands,
    mut player: Query<(&Transform, &mut Specials), With<Player>>,
    mut despawn_ev: EventWriter<DespawnEvent>,
    bullets: Query<(Entity, &ColliderType), With<Bullet>>,
    assets: Res<AssetServer>,
) {
    let Ok((&player, mut specials)) = player.get_single_mut() else {
        return;
    };

    if specials.get() == 0 {
        return;
    };

    specials.subtract(1);

    for (bullet, kind) in bullets.iter() {
        if *kind == ColliderType::EnemyBullet {
            despawn_ev.send(DespawnEvent::new(bullet, true).with_score(1));
        }
    }

    let sprite = MetaSprite {
        sprite: Sprite {
            custom_size: Some(1.5 * METRE_SQUARED),
            ..default()
        },
        texture: Some(assets.load("debug/sprites/up-arrow.png")),
        collider: Collider::cuboid(METRE * 0.5, METRE * 0.5),
        ..default()
    };

    let movement = Movement::new(
        Vec2::ZERO,
        Vec2::ZERO,
        true,
        Vec2::new(0.0, 30.0),
        Vec2::ZERO,
    );

    BulletGroup {
        formation: Formation::circular(false, 20.0),
        origin: player,
        number: 35,
        collider_type: ColliderType::PlayerBullet,
        bullet: Bullet::new(50.0, 50.0),
    }
    .spawn_all(&mut commands, movement, sprite);
}

pub fn spawn_player_bullet(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    input: Res<Input<KeyCode>>,
    dt: Res<Time>,
    mut cooldown: ResMut<PlayerAttackCD>,
    assets: Res<AssetServer>,
) {
    cooldown.tick(dt.delta());
    if !cooldown.finished() {
        return;
    }
    if !input.pressed(KeyCode::Z) {
        return;
    }

    let Ok(&player) = player.get_single() else {
        return;
    };
    let bullet_texture = assets.load("debug/sprites/up-arrow.png");
    let (bullet_speed_x, bullet_speed_y) = (5.0, 5.0);

    // The three bullets shot have different speeds and directions
    let attributes = [
        (Color::rgb(0.75, 0.25, 0.5), Vec2::new(0.0, bullet_speed_y)),
        (
            Color::rgb(0.75, 0.55, 0.5),
            Vec2::new(-bullet_speed_x, bullet_speed_y * 0.8),
        ),
        (
            Color::rgb(0.75, 0.55, 0.5),
            Vec2::new(bullet_speed_x, bullet_speed_y * 0.8),
        ),
    ];

    for (colour, velocity) in attributes {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: colour,
                    custom_size: Some(2. * METRE_SQUARED),
                    ..default()
                },
                texture: bullet_texture.clone(),
                transform: Transform {
                    translation: player.translation + Vec3::new(0.0, 5.0, 0.0),
                    ..default()
                },
                ..default()
            },
            Bullet::new(5.0, 20.0),
            RigidBody::Dynamic,
            Velocity::zero(),
            Movement::new(velocity, Vec2::ZERO, true, Vec2::ZERO, Vec2::new(0.0, 10.0)),
            Collider::cuboid(METRE / 2., METRE / 2.),
            ColliderType::PlayerBullet,
            ColliderType::PlayerBullet.collision_group(),
            SolverGroups::new(PLAYER_BULLET_COL, Group::NONE),
            Sensor,
        ));
    }
    cooldown.reset();
}

pub fn move_player(
    mut player: Query<(&mut Velocity, &Movement, &mut TextureAtlasSprite), With<Player>>,
    input: Res<Input<KeyCode>>,
    game_options: Res<crate::GameOptions>,
) {
    let Ok((mut rapier_vel, movement, mut sprite)) = player.get_single_mut() else { return; };

    // input.pressed() returns a boolean value, which can be converted into an integer,
    // as false = 0 and true = 1
    // Therefore, the Right arrow can be set as +1 when true and the Left arrow
    // as -1 when true (by adding a coefficient of -1). This can be represented on a number line
    // as -1 = left, 0 = none, 1 = right. By adding the two values we get the overall
    // movement desired. Holding down only the right key gives +1, holding the left key gives
    // -1, holding both gives 0 and not pressing either gives 0.
    // The same thing is implemented for vertical movement with the Up and Down arrows.
    let x = input.pressed(KeyCode::Right) as i8 - (input.pressed(KeyCode::Left) as i8);
    let y = input.pressed(KeyCode::Up) as i8 - (input.pressed(KeyCode::Down) as i8);

    // The focus value is the value to divide the velocity by if the player wants to slow down (focus)
    // Unlike the other inputs, this is dependent on a game setting, whether or not the player
    // wants to be focused by default or wants to hold down a key to become focused.
    // focus is calculated by adding 1 to the value of input.pressed(left_shift) if the
    // focus setting is left to normal. if the setting has been inverted, the value is calculated
    // by adding 1 to the negation of the same key. This is so that the focus is alway 1 or 2, and
    // never 0 to avoid divide-by-zero errors.
    let focus = if game_options.get_focus() {
        !input.pressed(KeyCode::LShift)
    } else {
        input.pressed(KeyCode::LShift)
    };

    let divisor: f32 = match focus {
        true => 1.8,
        false => 1.0,
    };

    // Update the player sprite depending on the direction they are moving.
    // The numbers 3, 4, and 5 correspond to indices of the texture atlas
    // for the player sprite, where 3 is moving left, 4 is moving right, and 5
    // is neither. The x value is checked to achieve this.
    sprite.index = if x == 1 {
        4
    } else if x == -1 {
        3
    } else {
        5
    };

    // Construct a 2D Vector from the x and y deltas
    let mut move_delta = Vec2::new(x as f32, y as f32);
    if move_delta != Vec2::ZERO {
        // Ensure that the vector is normalised to unit length
        move_delta /= move_delta.length();
    }

    // Update the physics simulation's velocity. This is done by multiplying
    // the movement delta above (i.e. direction vector) by the player speed
    // and dividing by the focus in order to slow down by a half when
    // the player is focusing.
    rapier_vel.linvel = move_delta * movement.velocity / divisor;
}
