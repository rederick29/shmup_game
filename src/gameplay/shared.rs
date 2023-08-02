use bevy::asset::Asset;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::fmt::{Debug, Display};

pub const METRE: f32 = 20.0;
pub const METRE_SQUARED: Vec2 = Vec2::new(METRE, METRE);

// Quick way of importing all of the physics-related items.
pub mod physics {
    pub use bevy_rapier2d::prelude::{
        ActiveEvents, Collider, CollisionEvent, CollisionGroups, Group, LockedAxes, RigidBody,
        Sensor, SolverGroups, Velocity,
    };
}

// Name marker. Allows naming entities and use in UI.
#[derive(Debug, Clone, Default, Component, Deref, DerefMut)]
pub struct Name<'a>(pub Option<&'a str>);

// Allow the compiler to convert from strings to 'Name' automatically
impl<'s> From<&'s str> for Name<'s> {
    fn from(string: &'s str) -> Self {
        if string.is_empty() {
            return Name(None);
        }
        Name(Some(string))
    }
}

// Generic counter implementation.
pub trait Counter {
    type Data: Display + Debug;

    fn set(&mut self, n: Self::Data);
    fn get(&self) -> Self::Data;
    fn add(&mut self, n: Self::Data);
    fn subtract(&mut self, n: Self::Data);
}

// Abstraction of entity movement.
#[derive(Component, Debug, PartialEq, Clone)]
pub struct Movement {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    /// `forward` is an alternative to the `velocity` and `acceleration` vectors
    /// which allows using a scalar value in the direction that the object is facing.
    pub local: bool,
    pub v_local: Vec2,
    pub a_local: Vec2,
    // Used for movement simulation for only the initial run.
    first_run: bool,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            local: false,
            ..Movement::ZERO
        }
    }
}

impl Movement {
    pub const fn new(
        velocity: Vec2,
        acceleration: Vec2,
        local: bool,
        v_local: Vec2,
        a_local: Vec2,
    ) -> Self {
        Self {
            velocity,
            acceleration,
            local,
            v_local,
            a_local,
            first_run: true,
        }
    }

    // Only sets the absolute velocity values
    pub fn absolute(velocity: Vec2, acceleration: Vec2) -> Self {
        Self {
            velocity,
            acceleration,
            ..default()
        }
    }

    // Only sets velocity values relative to the entity's rotation.
    pub fn relative(velocity: Vec2, acceleration: Vec2) -> Self {
        Self {
            local: true,
            v_local: velocity,
            a_local: acceleration,
            ..default()
        }
    }

    // All-zero movement data, with local set to true
    pub const ZERO: Movement = Movement {
        velocity: Vec2::ZERO,
        acceleration: Vec2::ZERO,
        local: true,
        v_local: Vec2::ZERO,
        a_local: Vec2::ZERO,
        first_run: true,
    };

    // Constant velocity of 1 going forward
    pub const ONE_FWD: Movement = Movement {
        velocity: Vec2::ZERO,
        acceleration: Vec2::ZERO,
        local: true,
        v_local: Vec2::new(0.0, 1.0),
        a_local: Vec2::ZERO,
        first_run: true,
    };
}

#[derive(Component)]
pub struct Health {
    pub total: f32,
    pub current: f32,
}
impl Health {
    pub fn new(max_health: f32, starting_health: Option<f32>) -> Self {
        if let Some(current) = starting_health {
            Self {
                total: max_health,
                current,
            }
        } else {
            Self {
                total: max_health,
                current: max_health,
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FormationShape {
    /// Requires radius.
    Circular,
    /// Requires radius, frequency and amplitude.
    /// Works correctly for small amplites only. Low frequency also recommended.
    Harmonic,
    /// Requires a target and the entity size.
    Linear,
    /// Requires a target.
    Positional,
}

// Bullet or enemy formation definition
#[derive(Debug, Clone)]
pub struct Formation {
    pub kind: FormationShape,
    // Should the formation be generated randomly or in order
    pub randomised: bool,
    // TODO: For generating spirals.
    pub ratio: Option<f32>,
    pub radius: Option<f32>,
    pub amplitude: Option<f32>,
    pub frequency: Option<f32>,
    pub target: Option<Transform>,
    pub entity_size: Option<Vec2>,
}

impl Default for Formation {
    fn default() -> Self {
        Self {
            kind: FormationShape::Circular,
            randomised: false,
            ratio: None,
            radius: Some(1.0),
            amplitude: None,
            frequency: None,
            target: None,
            entity_size: None,
        }
    }
}

impl Formation {
    pub fn circular(randomised: bool, radius: f32) -> Self {
        Self {
            kind: FormationShape::Circular,
            randomised,
            radius: Some(radius),
            ..default()
        }
    }

    pub fn harmonic(randomised: bool, radius: f32, amplitude: f32, frequency: f32) -> Self {
        Self {
            kind: FormationShape::Harmonic,
            randomised,
            radius: Some(radius),
            amplitude: Some(amplitude),
            frequency: Some(frequency),
            ..default()
        }
    }

    pub fn linear(target: Transform, entity_size: Vec2) -> Self {
        Self {
            kind: FormationShape::Linear,
            radius: None,
            target: Some(target),
            entity_size: Some(entity_size),
            ..default()
        }
    }

    #[allow(unused)]
    pub fn positional(target: Transform) -> Self {
        Self {
            kind: FormationShape::Positional,
            radius: None,
            target: Some(target),
            ..default()
        }
    }

    /// Both parameters should be of unit length.
    fn rotation(relative_pos: Vec3, forward_direction: Vec3) -> Quat {
        let angle = forward_direction.angle_between(relative_pos);

        // For a very small theta, theta is approximately 0, so no rotaition needed.
        // This means that very slight rotations are ignored
        if angle < 1e-7 {
            return Quat::IDENTITY;
        }

        let axis = forward_direction.cross(relative_pos);
        // Rotations of PI radians can be problematic. Handle this edge case.
        if axis.length() < 1e-7 {
            return Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI);
        }

        Quat::from_axis_angle(axis.normalize_or_zero(), angle)
    }
    /// `n`: n >= 1, n ∈ ℤ
    /// For circular formation, n is the number of vertices
    /// For harmonic formation, n is the phase out of 2 PI radians
    /// For linear formation, n is the position on the line
    /// `i`: Current iteration.
    pub fn transform(&self, i: u16, n: u16, origin: Transform) -> Transform {
        use std::f32::consts::TAU;
        match self.kind {
            FormationShape::Circular => {
                let radius = self
                    .radius
                    .expect("No radius was provided for a Circular formation!");

                // Position
                let mut theta = (TAU / n as f32) * i as f32;

                if self.randomised {
                    use rand::Rng;
                    theta = TAU * rand::thread_rng().gen::<f32>();
                }

                let translation = Vec3::new(
                    origin.translation.x + radius * theta.cos(),
                    origin.translation.y + radius * theta.sin(),
                    origin.translation.z,
                );

                // Rotation
                let relative_target_pos = (translation - origin.translation).normalize_or_zero();
                // NOTE: Changing this can allow for a variety of attacks. Pretty cool!
                let forward = Vec3::X;
                let rotation = Formation::rotation(relative_target_pos, forward);

                Transform {
                    translation,
                    rotation,
                    scale: origin.scale,
                }
            }
            FormationShape::Harmonic => {
                let radius = self
                    .radius
                    .expect("No radius was provided for a Harmonic formation!");
                let amplitude = self
                    .amplitude
                    .expect("No amplitude was provided for a Harmonic formation!");
                let frequency = self
                    .frequency
                    .expect("No frequency was provided for a Harmonic formation!");

                // 2*pi*f
                let angular_speed = TAU * frequency;
                let time = i as f32 / n as f32;
                // x = A * cos(2*pi*f*t)
                let displacement = amplitude * (angular_speed * time).cos();
                // x / r = theta in radians
                let theta = displacement / radius;

                // The following is for a -y-hanging pendulum
                // in order to rotate this, there needs to be two coefficients, a and b, i.e., :
                // a * sin(theta) * radius + b * origin.translation.x
                // for simple PI/2 rotations, the coefficients can be 1 and -1 in any combination.

                let translation = Vec3::new(
                    theta.sin() * radius + origin.translation.x,
                    -(theta.cos()) * radius + origin.translation.y,
                    0.0,
                );

                let relative_pos = (translation - origin.translation).normalize_or_zero();
                let forward = Vec3::Y;
                let rotation = Formation::rotation(relative_pos, forward);

                Transform {
                    translation,
                    rotation,
                    scale: origin.scale,
                }
            }
            FormationShape::Linear => {
                let target = self
                    .target
                    .expect("No target was provided for a Linear formation!");
                let entity_size = self
                    .entity_size
                    .expect("No entity size was provided for a Linear formation!");

                let relative_pos = target.translation - origin.translation;
                let direction = relative_pos.normalize_or_zero();
                Transform {
                    translation: origin.translation
                        + direction * entity_size.extend(0.0) * i as f32,
                    rotation: Formation::rotation(direction, Vec3::Y),
                    scale: origin.scale,
                }
            }
            FormationShape::Positional => {
                let target = self
                    .target
                    .expect("No target was provided for a Positional formation!");

                let relative_pos = target.translation - origin.translation;
                let direction = relative_pos.normalize_or_zero();
                Transform {
                    translation: origin.translation + direction,
                    rotation: Formation::rotation(direction, Vec3::Y),
                    scale: origin.scale,
                }
            }
        }
    }
}

// Generic interface for sprites information
pub trait ExtraSpriteInfo {
    type C: Bundle;
    type S: Clone;
    type T: Asset + Clone;

    fn texture(&self) -> Option<Handle<Self::T>>;
    fn sprite(&self) -> Self::S;
    fn collider(&self) -> Collider;
    fn grazing_collider(&self) -> Option<Collider> { None }
    fn bundle(&self, transform: Transform) -> Self::C;
}

// A better collection of data to go with a Sprite
#[derive(Debug, Clone)]
pub struct MetaSprite {
    pub sprite: Sprite,
    pub texture: Option<Handle<Image>>,
    pub collider: Collider,
    pub grazing_collider: Option<Collider>,
}

impl ExtraSpriteInfo for MetaSprite {
    type C = SpriteBundle;
    type S = Sprite;
    type T = Image;

    fn texture(&self) -> Option<Handle<Self::T>> {
        self.texture.clone()
    }

    fn sprite(&self) -> Self::S {
        self.sprite.clone()
    }

    fn collider(&self) -> Collider {
        self.collider.clone()
    }

    fn grazing_collider(&self) -> Option<Collider> {
        self.grazing_collider.clone()
    }

    fn bundle(&self, transform: Transform) -> Self::C {
        Self::C {
            sprite: self.sprite(),
            texture: self.texture().unwrap_or_default(),
            transform,
            ..default()
        }
    }
}

// A better collection of data to go with an Atlas
#[derive(Debug, Clone)]
pub struct MetaSpriteAtlas {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Option<Handle<TextureAtlas>>,
    pub collider: Collider,
    pub grazing_collider: Option<Collider>,
}

impl ExtraSpriteInfo for MetaSpriteAtlas {
    type C = SpriteSheetBundle;
    type S = TextureAtlasSprite;
    type T = TextureAtlas;

    fn texture(&self) -> Option<Handle<Self::T>> {
        self.texture_atlas.clone()
    }

    fn sprite(&self) -> Self::S {
        self.sprite.clone()
    }

    fn collider(&self) -> Collider {
        self.collider.clone()
    }

    fn grazing_collider(&self) -> Option<Collider> {
        self.grazing_collider.clone()
    }

    fn bundle(&self, transform: Transform) -> Self::C {
        Self::C {
            sprite: self.sprite(),
            texture_atlas: self.texture().unwrap_or_default(),
            transform,
            ..default()
        }
    }
}

impl Default for MetaSprite {
    fn default() -> Self {
        Self {
            sprite: Sprite::default(),
            texture: None,
            collider: Collider::default(),
            grazing_collider: None,
        }
    }
}

impl Default for MetaSpriteAtlas {
    fn default() -> Self {
        Self {
            sprite: TextureAtlasSprite::default(),
            texture_atlas: None,
            collider: Collider::default(),
            grazing_collider: None,
        }
    }
}

// Simulate an entity's movement based on its Movement component
// This does not modify the transform directly but rather updates
// the physics simulation's velocity data.
pub fn move_object<T: Component>(
    mut object: Query<(&mut Velocity, &mut Movement, &Transform), With<T>>,
    dt: Res<Time>,
) {
    for (mut rapier_vel, mut movement, transform) in &mut object {
        // Reborrow movement because compiler thinks I'm borrowing
        // movement immutably and mutably simultaneosly.
        let movement = &mut *movement;

        // working value for change in velocity
        let mut dv = Vec2::ZERO;

        if movement.local {
            // Update velocity due to acceleration
            movement.v_local += movement.a_local * dt.delta_seconds();
            // Convert the relative velocity to absolute velocity
            // by extending it into the 3rd dimension and multiplying by
            // the rotation quaternion of the entity. Then truncate back into
            // 2 dimensions and add it to the working value (total change).
            dv += (transform.rotation * movement.v_local.extend(0.0)).truncate();
        }

        // Accelerate
        movement.velocity += movement.acceleration * dt.delta_seconds();
        dv += movement.velocity;

        // Update the physics simulation with the working value of linear
        // velocity created above. Make all movement operations in metres.
        rapier_vel.linvel = dv * METRE;
    }
}

/// T is a number between 0 and 1 which indicates how far between the points to go. 0.5 is the
/// midpoint
#[allow(unused)]
pub fn lerp(point1: Vec2, point2: Vec2, t: f32) -> Vec2 {
    point1 + (point2 - point1) * t
}

pub fn magnetise_to(entity_movement: &mut Movement, entity_transform: &Transform, target_transform: &Transform, strength: f32, accel_mode: bool) {
    let displacement = target_transform.translation - entity_transform.translation;
    let direction = displacement.normalize().truncate();

    let variable = if accel_mode {
        &mut entity_movement.acceleration
    } else {
        &mut entity_movement.velocity
    };

    *variable = Vec2::splat(strength) * direction;
}
