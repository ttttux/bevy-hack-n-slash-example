#![feature(exact_size_is_empty)]

use std::time::Duration;
use bevy::prelude::*;
use bevy_framepace::*;

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, player_movement_system)
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("FREE_Samurai 2D Pixel Art v1.2/Sprites/IDLE.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 10, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 9 };
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_scale(Vec3::splat(6.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn player_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Transform)>,
    input: Res<ButtonInput<KeyCode>>,

) {
    if query.iter().is_empty() {
        return;
    }

    let (mut player, mut transform) = query.single_mut();

    let mut movement_x = 0.0;
    let mut movement_y = 0.0;

    if input.pressed(KeyCode::ArrowLeft) {
        movement_x -= 1.0;
    }

    if input.pressed(KeyCode::ArrowRight) {
        movement_x += 1.0;
    }

    if input.pressed(KeyCode::ArrowUp) {
        movement_y += 1.0;
    }

    if input.pressed(KeyCode::ArrowDown) {
        movement_y -= 1.0;
    }

    let movement_distance_x = movement_x * 500.0 * time.delta_secs();
    let movement_distance_y = movement_y * 500.0 * time.delta_secs();
    transform.translation.y += movement_distance_y;
    transform.translation.x += movement_distance_x;

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}
