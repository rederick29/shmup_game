use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_hanabi::prelude::*;

use super::GameplayState;

// Hash table holding handles to loaded texture atlases.
// The keys are strings/names, while the values are the handles.
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct Atlases<'a>(HashMap<&'a str, Handle<TextureAtlas>>);

// 'static lifetime is required for all the hash tables, as the strings
// must be available in memory for the entire duration of the game running.

pub fn load_texture_atlases(
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<Atlases<'static>>,
    asset_server: Res<AssetServer>,
) {
    // Define list of real images to be loaded as texture atlases by
    // giving their path relative to the game directory, the size of one texture
    // in (x, y) pixels, the number of columns and number of rows in the texture atlas.
    let assets = [
        ("sprites/white-plane3.png", Vec2::new(60.0, 90.0), 8, 1),
        ("sprites/enemy-projectile.png",Vec2::new(128.0, 128.0),4,1),
        ("sprites/enemy-small.png", Vec2::new(64.0, 64.0), 2, 1),
        ("sprites/enemy-medium.png", Vec2::new(128.0, 64.0), 2, 1),
        ("sprites/enemy-big.png", Vec2::new(120.0, 128.0), 2, 1),
    ];
    for (path, size, columns, rows) in assets {
        // Load atlas
        let texture_atlas =
            TextureAtlas::from_grid(asset_server.load(path), size, columns, rows, None, None);
        // Add atlas into the Atlases resource for later use
        atlas_handles.insert(path, texture_atlases.add(texture_atlas));
    }
}

// Hash table holding handles to loaded particle effects.
// The keys are strings/names, while the values are the handles.
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct ParticleEffects<'a>(HashMap<&'a str, Handle<EffectAsset>>);

pub fn load_particle_effects(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut effect_handles: ResMut<ParticleEffects<'static>>,
) {
    // Define and add the particle effect for the player rocket booster
    let mut module = Module::default();

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(1.0));
    let update_accel = AccelModifier::new(module.lit(Vec3::Y * -8.0));
    let pos_b_r = module.lit(40.0);
    let pos_t_r = module.lit(0.0);
    let pos_h = module.lit(50.0);
    let vel_c = module.lit(Vec3::ZERO);
    let vel_s = module.lit(1.0);
    let vel_a = module.lit(Vec3::Z);

    let player_booster_effect = effects.add(
        EffectAsset::new(8192, Spawner::rate(CpuValue::Single(150.0)), module)
        .with_name("player_booster")
        .with_property(
            "acceleration",
            graph::Value::Vector(Vec3::new(0.0, -3.0, 0.0).into()),
        )
        .init(SetPositionCone3dModifier {
            base_radius: pos_b_r,
            top_radius: pos_t_r,
            height: pos_h,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocityCircleModifier {
            center: vel_c,
            speed: vel_s,
            axis: vel_a,
        })
        .init(init_lifetime)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec4::splat(1.0));
                gradient.add_key(0.1, Vec4::new(1.0, 1.0, 0.0, 1.0));
                gradient.add_key(0.4, Vec4::new(1.0, 0.0, 0.0, 1.0));
                gradient.add_key(1.0, Vec4::splat(0.0));
                gradient
            },
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec2::splat(6.0));
                gradient.add_key(0.5, Vec2::splat(8.0));
                gradient.add_key(0.8, Vec2::splat(4.8));
                gradient.add_key(1.0, Vec2::splat(3.0));
                gradient
            },
            screen_space_size: false,
        }),
    );
    effect_handles.insert("player_booster", player_booster_effect);
}

// Resource holding a single handle for the loaded background image.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct BackgroundHandle(pub Handle<Image>);

pub fn load_background(asset_server: Res<AssetServer>, mut background: ResMut<BackgroundHandle>) {
    let bg = asset_server.load("backgrounds/level_1.png");
    background.0 = bg;
}

// Check the load status of all the assets during the loading stage. These functions are used
// to check when it is ok to switch game states (when everything has loaded).
pub fn check_background_loaded(asset_server: Res<AssetServer>, bg: Res<BackgroundHandle>) -> bool {
    if asset_server.get_load_state(&bg.0) != bevy::asset::LoadState::Loaded {
        return false;
    }
    true
}

#[allow(unused)] // bug: hangs
pub fn check_atlases_loaded(
    asset_server: Res<AssetServer>,
    atlases: Res<Atlases<'static>>,
) -> bool {
    info!("still loading atlases...");
    atlases
        .values()
        .all(|v| asset_server.get_load_state(v) == LoadState::Loaded)
}

#[allow(unused)] // bug: hangs
pub fn check_particles_loaded(
    asset_server: Res<AssetServer>,
    particles: Res<ParticleEffects<'static>>,
) -> bool {
    particles
        .values()
        .all(|v| asset_server.get_load_state(v) == LoadState::Loaded)
}

// Continue into the playing game state.
pub fn finish_loading(mut next_state: ResMut<NextState<GameplayState>>) {
    info!("Finished GameplayState::Loading");
    next_state.set(GameplayState::Playing);
}
