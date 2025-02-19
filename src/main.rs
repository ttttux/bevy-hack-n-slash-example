#![feature(exact_size_is_empty)]
#![feature(portable_simd)]

use bevy::prelude::Color;
use bevy_framepace::*;
use bevy::prelude::*;

use std::simd::f32x2;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use bevy::color::palettes::css::RED;
use bevy::input::common_conditions::input_pressed;
use bevy::input::keyboard::Key::ColorF0Red;
use bevy::prelude::*;
use bevy::prelude::KeyCode::ArrowLeft;
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
        .add_systems(Update, enemy_movement_system)
        .add_systems(Update, animate_enemy)
        .run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Attack1,
    Idle,
    Run,
    Hurt,
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
    animation_state: AnimationState,
    last_attack: u64,
}

#[derive(Component)]
struct SwoardHitbox {
    position: Vec3,
    rect: Rectangle,
}

#[derive(Component)]
struct StdEnemy {
    movement_speed: f32,
    position: Vec3,
    direction: Direction,
    animation_state: AnimationState,
    last_attack: u64,
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite, &mut Player)>,
) {
    for (indices, mut timer, mut sprite, mut player) in &mut query {
        if player.animation_state == AnimationState::Idle {
            Timer::from_seconds(0.3, TimerMode::Repeating);
        } else if player.animation_state == AnimationState::Attack1 {
            Timer::from_seconds(0.2, TimerMode::Repeating);
        } else if player.animation_state == AnimationState::Run {
            Timer::from_seconds(0.3, TimerMode::Repeating);
        }

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

fn animate_enemy(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite, &mut StdEnemy)>,
) {
    if query.iter().is_empty() {
        return;
    }

    for (indices, mut timer, mut sprite, mut enemy) in &mut query {
        if enemy.animation_state == AnimationState::Idle {
            Timer::from_seconds(0.3, TimerMode::Repeating);
        } else if enemy.animation_state == AnimationState::Attack1 {
            Timer::from_seconds(0.2, TimerMode::Repeating);
        } else if enemy.animation_state == AnimationState::Run {
            Timer::from_seconds(0.3, TimerMode::Repeating);
        }

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

        if enemy.direction == Direction::Left {
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
            animation_state: AnimationState::Idle,
            last_attack: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    ));
    setup_enemy(commands, asset_server, texture_atlas_layouts);
}

fn setup_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("FreeNinja/YellowNinja/yellowNinja - idle.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(128), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 7 };

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
        StdEnemy {
            movement_speed: 500.0f32,
            position: Vec3::new(200.0, 0.0, 0.0),
            direction: Direction::Left,
            animation_state: AnimationState::Idle,
            last_attack: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    ));
}

fn enemy_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut StdEnemy, &mut Sprite, &mut Transform, &mut AnimationIndices)>,
    mut player_query: Query<(&mut Player)>,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("FreeNinja/YellowNinja/yellowNinja - walk.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(128), 10, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    if query.iter().is_empty() {
        return;
    }

    if player_query.iter().is_empty() {
        return;
    }

    let mut player = player_query.single_mut();

    let (mut enemy, mut sprite, mut transform, mut indices) = query.single_mut();

    let mut movement_x = 0.0;
    let mut movement_y = 0.0;

    if enemy.animation_state == AnimationState::Attack1 {
        return;
    }

    if player.position.x > enemy.position.x {
        /*if (enemy.position.x + player.position.x) < 0.25 {
            enemy.direction = Direction::Right;
            return;
        }*/

        movement_x += 1.0;
        enemy.direction = Direction::Right;

        if enemy.animation_state != AnimationState::Run {
            sprite.image = texture;
            sprite.texture_atlas = Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
                ..default()
            });
            indices.first = 0;
            indices.last = 9;
            enemy.animation_state = AnimationState::Run;
        }
    } else if player.position.x < enemy.position.x {
        /*if (player.position.x + enemy.position.x) > 0.25 {
            enemy.direction = Direction::Left;
            return;
        }*/

        movement_x -= 1.0;
        enemy.direction = Direction::Left;

        if enemy.animation_state != AnimationState::Run {
            sprite.image = texture;
            sprite.texture_atlas = Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
                ..default()
            });
            indices.first = 0;
            indices.last = 9;
            enemy.animation_state = AnimationState::Run;
        }
    }

    if !(player.position.x < enemy.position.x || player.position.x > enemy.position.x) && enemy.animation_state != AnimationState::Idle {
        let texture = asset_server.load("FreeNinja/YellowNinja/yellowNinja - idle.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(128), 8, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        sprite.image = texture;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: texture_atlas_layout,
            ..default()
        });
        indices.first = 0;
        indices.last = 7;

        enemy.animation_state = AnimationState::Idle;
    }

    let movement_distance_x = movement_x * 250.0 * time.delta_secs();
    let movement_distance_y = movement_y * 250.0 * time.delta_secs();
    transform.translation.y += movement_distance_y;
    transform.translation.x += movement_distance_x;
    enemy.position = transform.translation;

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn player_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Sprite, &mut Transform, &mut AnimationIndices)>,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("FREE_Samurai 2D Pixel Art v1.2/Sprites/RUN.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 16, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    if query.iter().is_empty() {
        return;
    }

    let (mut player, mut sprite, mut transform, mut indices) = query.single_mut();

    let mut movement_x = 0.0;
    let mut movement_y = 0.0;

    if player.animation_state == AnimationState::Attack1 {
        return;
    }

    if input.pressed(KeyCode::ArrowRight) {
        movement_x += 1.0;
        player.direction = Direction::Right;

        if player.animation_state != AnimationState::Run {
            sprite.image = texture;
            sprite.texture_atlas = Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
                ..default()
            });
            indices.first = 0;
            indices.last = 15;
            player.animation_state = AnimationState::Run;
        }
    } else if input.pressed(KeyCode::ArrowLeft) {
        movement_x -= 1.0;
        player.direction = Direction::Left;

        if player.animation_state != AnimationState::Run {
            sprite.image = texture;
            sprite.texture_atlas = Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
                ..default()
            });
            indices.first = 0;
            indices.last = 15;
            player.animation_state = AnimationState::Run;
        }
    }

    if input.pressed(KeyCode::ArrowUp) {
        movement_y += 1.0;
    }

    if input.pressed(KeyCode::ArrowDown) {
        movement_y -= 1.0;
    }

    if !(input.pressed(KeyCode::ArrowRight) || input.pressed(ArrowLeft)) && player.animation_state != AnimationState::Idle {
        let texture = asset_server.load("FREE_Samurai 2D Pixel Art v1.2/Sprites/IDLE.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 10, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        sprite.image = texture;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: texture_atlas_layout,
            ..default()
        });
        indices.first = 0;
        indices.last = 9;

        player.animation_state = AnimationState::Idle;
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

    if input.pressed(KeyCode::KeyE) && SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - player.last_attack >= 1 {
        let color: Color = bevy::prelude::Color::from(RED);
        let rect = meshes.add(Rectangle::new(96.0, 96.0));

        commands.spawn((
            Mesh2d(rect),
            //MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(transform.translation.x + 96.0/2.0, transform.translation.y - 96.0/2.0, 0.0),
        ));

        sprite.image = texture;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: texture_atlas_layout,
            index: 3,
            ..default()
        });
        indices.first = 0;
        indices.last = 5;

        player.animation_state = AnimationState::Attack1;
        player.last_attack = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    } else if player.animation_state == AnimationState::Attack1 && sprite.clone().texture_atlas.unwrap().index == 5 {
        let texture = asset_server.load("FREE_Samurai 2D Pixel Art v1.2/Sprites/IDLE.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 10, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        sprite.image = texture;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: texture_atlas_layout,
            ..default()
        });
        indices.first = 0;
        indices.last = 9;

        player.animation_state = AnimationState::Idle;
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