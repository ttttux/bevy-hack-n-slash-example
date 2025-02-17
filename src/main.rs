#![feature(exact_size_is_empty)]
#![feature(portable_simd)]

use bevy::prelude::Color;
use bevy_framepace::*;
use bevy::prelude::*;

use std::simd::f32x2;
use std::time::Duration;
use bevy::color::palettes::css::RED;
use bevy::input::common_conditions::input_pressed;
use bevy::input::keyboard::Key::ColorF0Red;
use bevy::prelude::*;
use bevy::reflect::Enum;
use bevy::render::mesh::RectangleMeshBuilder;
use bevy::render::render_resource::AsBindGroupShaderType;
use bevy::text::cosmic_text::ShapeGlyph;
use bevy::text::cosmic_text::SwashContent::Color as OtherColor;
use bevy_framepace::*;
use crate::Direction::Left;

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, player_movement_system)
        .add_systems(Update, player_attack_system)
        .add_systems(Update, remove_tmp_components)
        .run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Player {
    movement_speed: f32,
    position: Vec3,
    direction: Direction,
}

#[derive(Component)]
struct SwoardHitbox {
    position: Vec3,
    rect: Rectangle,
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite, &mut Player)>,
) {
    for (indices, mut timer, mut sprite, mut player) in &mut query {
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

        if player.direction == Direction::Left {
            sprite.flip_x = true;
        } else {
            sprite.flip_x = false;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut settings: ResMut<FramepaceSettings>,
) {
    settings.limiter = Limiter::from_framerate(60.0);

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
        Transform::from_scale(Vec3::splat(3.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player {
            movement_speed: 500.0f32,
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Direction::Right,
        }
    ));
}

fn player_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Sprite, &mut Transform)>,
    input: Res<ButtonInput<KeyCode>>,

) {
    if query.iter().is_empty() {
        return;
    }

    let (mut player, mut sprite, mut transform) = query.single_mut();

    let mut movement_x = 0.0;
    let mut movement_y = 0.0;

    if input.pressed(KeyCode::ArrowLeft) {
        movement_x -= 1.0;
        player.direction = Direction::Left;
    }

    if input.pressed(KeyCode::ArrowRight) {
        movement_x += 1.0;
        player.direction = Direction::Right;
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
    player.position = transform.translation;

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn player_attack_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Sprite, &mut Transform, &mut AnimationIndices)>,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if query.iter().is_empty() {
        return;
    }

    let texture = asset_server.load("FREE_Samurai 2D Pixel Art v1.2/Sprites/ATTACK 1.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 6, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let (mut player, mut sprite, mut transform, mut indices) = query.single_mut();

    if input.pressed(KeyCode::KeyE) {
        let color: Color = bevy::prelude::Color::from(RED);
        let rect = meshes.add(Rectangle::new(96.0, 96.0));

        commands.spawn((
            Mesh2d(rect),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(transform.translation.x + 96.0/2.0, transform.translation.y - 96.0/2.0, 0.0),
        ));

        sprite.image = texture;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: texture_atlas_layout,
            ..default()
        });
        indices.last = 5;
    }
}

fn remove_tmp_components(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Mesh2d, &mut Transform)>,
) {
    if !query.is_empty() {
        let (entity, mut mesh, mut transform) = query.single_mut();

        commands.entity(entity).despawn();
    }
}