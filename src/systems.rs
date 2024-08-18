use bevy::input::keyboard::Key;
use bevy::input::mouse::{self, MouseMotion};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use bevy::ui::{AlignItems, JustifyContent, Val, UiRect, Style};
use crate::components::{Ability, CooldownUi, HealthText, Line, Invulnerability, ScoreText, Score, Points, PointMarker, PauseMenu, MousePosition , Lifetime, Cooldowns, DirectionComponent, MovementSpeed, CollisionBox, Player, Tag};
use crate::events::{CollisionEvent};
use crate::{GameState, MAP_SPIRITE};
use rand::Rng;
use std::f32::consts::PI;
use std::time::Duration;

use crate::{EnemyCount, GameTextures, MouseCoords, ENEMY_SPRITE, LINE_SPRITE, PLAYER_SPRITE};
// Systems Implementation

pub fn camera_follow_player(
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,             // Query to get the player's position
        Query<&mut Transform, With<Camera2d>>,       // Query to get the camera's Transform
    )>,
    window_query: Query<&Window>,                    // Query to get the window
) {
    // First, get the player's Transform
    let player_position = {
        if let Ok(player_transform) = param_set.p0().get_single() {
            Some(player_transform.translation)
        } else {
            None
        }
    };

    // If we have the player's position, continue
    if let Some(player_position) = player_position {
        // Then, get the window dimensions
        if let Ok(window) = window_query.get_single() {
            // Now we can safely get the camera's Transform
            if let Ok(mut camera_transform) = param_set.p1().get_single_mut() {
                let half_width = window.width() / 2.0;
                let half_height = window.height() / 2.0;

                // Calculate world bounds, ensuring min < max
                let min_x = -500.0 + half_width;
                let max_x = 500.0 - half_width;
                let min_y = -500.0 + half_height;
                let max_y = 500.0 - half_height;

                // Ensure bounds are valid (min should be less than max)
                if min_x < max_x && min_y < max_y {
                    // Calculate camera position with clamping
                    let camera_x = player_position.x.clamp(min_x, max_x);
                    let camera_y = player_position.y.clamp(min_y, max_y);

                    camera_transform.translation.x = camera_x;
                    camera_transform.translation.y = camera_y;
                } else {
                    // Handle the edge case where bounds are not valid (e.g., world is smaller than window)
                    camera_transform.translation.x = player_position.x;
                    camera_transform.translation.y = player_position.y;
                }
            }
        }
    }
}

pub fn enemy_killed(score: &mut ResMut<Score>, mut player: &mut Player, cooldowns_query: &mut Query<&mut Cooldowns>,) {
    score.increment();
    player.heal(1);
    println!("Score: {}", score.get_enemies_killed());
     // Apply cooldown reduction to all abilities
     // Apply cooldown reduction to all abilities
    for mut cooldowns in cooldowns_query.iter_mut() {
        for timer in cooldowns.cooldowns.values_mut() {
            // Reduce each ability's cooldown by 0.05 seconds, but ensure it doesn't go below zero
            let remaining_time = timer.duration().as_secs_f32();
            if remaining_time > 0.05 {
                let new_duration = remaining_time - 0.05;
                timer.set_duration(Duration::from_secs_f32(new_duration));
            } else {
                timer.set_duration(Duration::from_secs_f32(0.0)); // Set to zero if it's below 0.05 seconds
            }
        }
    }
}

pub fn display_score(_score: Res<Score>) {
    //println!("Enemies killed: {}", score.get_enemies_killed());
}

pub fn check_collisions(
    mut player_query: Query<(Entity, &mut Player, &CollisionBox, &Transform, Option<Mut< Invulnerability>>)>,    
    other_entities_query: Query<(Entity, &Transform, &CollisionBox), (Without<Player>, Without<Line>)>,
    mut collision_events: EventWriter<CollisionEvent>,
    points_query: Query<(Entity, &Transform), With<PointMarker>>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    mut points: ResMut<Points>,  
    mut despawned_entities: Local<HashSet<Entity>>,  // Track despawned entities
    line_query: Query<(Entity, &Transform, &CollisionBox), With<Line>>,
    mut exit: EventWriter<AppExit>, // Add the AppExit event writer
    mut cooldowns_query: Query<&mut Cooldowns>,
) {
    for (enemy_entity, transform, bounding_box) in other_entities_query.iter() {
        let enemy_min_x = transform.translation.x - bounding_box.width / 2.0;
        let enemy_max_x = transform.translation.x + bounding_box.width / 2.0;
        let enemy_min_y = transform.translation.y - bounding_box.height / 2.0;
        let enemy_max_y = transform.translation.y + bounding_box.height / 2.0;
        for (entity, mut player, player_box, player_transform, mut invulnerability_option) in player_query.iter_mut() {
        for (point_entity, point_transform) in points_query.iter() {
            let point = Vec2::new(point_transform.translation.x, point_transform.translation.y);

            if point.x > enemy_min_x
                && point.x < enemy_max_x
                && point.y > enemy_min_y
                && point.y < enemy_max_y
            {
                // Call the kill_enemy function
                enemy_killed(&mut score,&mut player, &mut cooldowns_query);
                

                // Despawn the enemy
                commands.entity(enemy_entity).despawn();
                break;
            }
        }
    }}
    for (attack_entity, attack_transform, attack_box) in line_query.iter() {
        let attack_min_x = attack_transform.translation.x - attack_box.width / 2.0;
        let attack_max_x = attack_transform.translation.x + attack_box.width / 2.0;
        let attack_min_y = attack_transform.translation.y - attack_box.height / 2.0;
        let attack_max_y = attack_transform.translation.y + attack_box.height / 2.0;

        for (enemy_entity, enemy_transform, enemy_box) in other_entities_query.iter() {
            let enemy_min_x = enemy_transform.translation.x - enemy_box.width / 2.0;
            let enemy_max_x = enemy_transform.translation.x + enemy_box.width / 2.0;
            let enemy_min_y = enemy_transform.translation.y - enemy_box.height / 2.0;
            let enemy_max_y = enemy_transform.translation.y + enemy_box.height / 2.0;

            if attack_max_x > enemy_min_x
                && attack_min_x < enemy_max_x
                && attack_max_y > enemy_min_y
                && attack_min_y < enemy_max_y
            {
                // Call the kill_enemy function
                score.increment();

                // Despawn the enemy
                commands.entity(enemy_entity).despawn();

                // Despawn the line after it collides with an enemy
                //commands.entity(line_entity).despawn();
                break;
            }
        }
    }

    
    for (entity, mut player, player_box, player_transform, mut invulnerability_option) in player_query.iter_mut() {
        let player_min_x = player_transform.translation.x - player_box.width / 2.0;
        let player_max_x = player_transform.translation.x + player_box.width / 2.0;
        let player_min_y = player_transform.translation.y - player_box.height / 2.0;
        let player_max_y = player_transform.translation.y + player_box.height / 2.0;

        if let Some(ref mut invulnerability) = invulnerability_option {
            if invulnerability.is_active() {
                continue; // Skip damage application if invulnerable
            }
        }

        for (enemy_entity, enemy_transform, enemy_box) in other_entities_query.iter() {
            let enemy_min_x = enemy_transform.translation.x - enemy_box.width / 2.0;
            let enemy_max_x = enemy_transform.translation.x + enemy_box.width / 2.0;
            let enemy_min_y = enemy_transform.translation.y - enemy_box.height / 2.0;
            let enemy_max_y = enemy_transform.translation.y + enemy_box.height / 2.0;

            if player_max_x > enemy_min_x
                && player_min_x < enemy_max_x
                && player_max_y > enemy_min_y
                && player_min_y < enemy_max_y
            {

            if player_max_x > enemy_min_x
                && player_min_x < enemy_max_x
                && player_max_y > enemy_min_y
                && player_min_y < enemy_max_y
            {
                // Handle collision, but only if player is not invulnerable
                player.take_damage(100, entity, &mut commands, invulnerability_option.as_deref_mut(), 0.5, &mut exit);
            }
        }
    }
}
}


pub fn handle_escape_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut curr_state: ResMut<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("gaming");
        if *curr_state.get() == GameState::Running {
            state.set(GameState::Paused);
        } else if *curr_state.get() == GameState::Paused {
            state.set(GameState::Running);
        }
    }
}

pub fn flicker_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Invulnerability), With<Player>>,
) {
    for (mut sprite, mut invulnerability) in query.iter_mut() {
        // Update the timer for invulnerability
        invulnerability.timer.tick(time.delta());

        // If the player is invulnerable, adjust the alpha value to create a flicker effect
        if invulnerability.is_active() {
            // Flicker by adjusting alpha value between 0.2 and 1.0
            let flicker_phase = (invulnerability.timer.elapsed_secs() * 10.0).sin();
            let new_alpha = 0.5 * flicker_phase.abs();

            // Directly set the alpha using set_alpha
            sprite.color.set_alpha(new_alpha);
        } else {
            // Ensure the player is fully visible when not invulnerable
            sprite.color.set_alpha(1.0);
        }
    }
}

// System to update the MousePosition resource whenever the mouse moves
pub fn update_mouse_position(
    q_windows: Query<&Window>,
    mut mouse_position: ResMut<MouseCoords>,
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera2d>>,
) {
    let window = q_windows.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok((camera_transform, projection)) = camera_query.get_single() {
            // Convert the cursor position to NDC (Normalized Device Coordinates)
            let window_size = Vec2::new(window.width(), window.height());
            let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;

            // Use the orthographic projection's area to convert NDC to world coordinates
            let world_position = camera_transform.translation()
                + Vec3::new(
                    ndc.x * projection.area.width() / 2.0,
                    -ndc.y * projection.area.height() / 2.0,
                    0.0,
                );

            mouse_position.x = world_position.x;
            mouse_position.y = world_position.y;

            //println!("Mouse Position in World: ({}, {})", mouse_position.x, mouse_position.y);
        }
    }
}

pub fn manage_invulnerability(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invulnerability)>,
) {
    for (entity, mut invulnerability) in query.iter_mut() {
        invulnerability.timer.tick(time.delta());
        println!(
            "Invulnerability timer ticking for entity {:?}, remaining: {:.2}",
            entity,
            invulnerability.timer.remaining_secs()
        );
        if invulnerability.timer.finished() {
            println!("Invulnerability expired for entity {:?}", entity);
            commands.entity(entity).remove::<Invulnerability>(); // Remove the component when the timer is done
        }
    }
}

pub fn update_cooldowns(
    time: Res<Time>,
    mut query: Query<&mut Cooldowns>,
) {
    for mut cooldowns in query.iter_mut() {
        for timer in cooldowns.cooldowns.values_mut() {
            timer.tick(time.delta());
        }
    }
}

pub fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.finished() {
            commands.entity(entity).despawn();  // This command is deferred and will execute later
        }
    }
}

pub fn update_cooldowns_ui(
    time: Res<Time>,
    mut cooldowns_query: Query<&mut Cooldowns>,
    mut text_query: Query<&mut Text, With<CooldownUi>>,
) {
    if let Ok(mut cooldowns) = cooldowns_query.get_single_mut() {
        // Update the UI text for each ability
        for (i, mut text) in text_query.iter_mut().enumerate() {
            let ability_text = match i {
                0 => format_cooldown_text("Attack", cooldowns.get_cooldown(Ability::Attack)),
                1 => format_cooldown_text("Ranged", cooldowns.get_cooldown(Ability::Ranged)),
                2 => format_cooldown_text("Dash", cooldowns.get_cooldown(Ability::Dash)),
                3 => format_cooldown_text("Aoe", cooldowns.get_cooldown(Ability::Aoe)),
                _ => "Unknown Ability".to_string(),
            };

            text.sections[0].value = ability_text;
        }
    }
}

fn format_cooldown_text(name: &str, cooldown: Option<f32>) -> String {
    let display_time = cooldown.unwrap_or(0.0);
    let display_time = if display_time > 0.0 {
        display_time
    } else {
        0.0
    };
    format!("{}: {:.1}s", name, display_time)
}

pub fn update_ui_text(
    player_query: Query<&Player>,
    score: Res<Score>,
    mut text_query: Query<(&mut Text, Option<&HealthText>, Option<&ScoreText>)>,
) {
    if let Ok(player) = player_query.get_single() {
        for (mut text, health_text, score_text) in text_query.iter_mut() {
            if health_text.is_some() {
                text.sections[0].value = format!("Health: {}", player.health);
            } else if score_text.is_some() {
                text.sections[0].value = format!("Score: {}", score.get_enemies_killed());
            }
        }
    }
}

pub fn show_pause_menu(mut query: Query<&mut Visibility, With<PauseMenu>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

pub fn hide_pause_menu(mut query: Query<&mut Visibility, With<PauseMenu>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

pub fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(), // Semi-transparent background
            visibility: Visibility::Hidden, // Initially hidden
            ..Default::default()
        },
        PauseMenu,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text::from_section(
                "Game Paused\nPress Esc to Resume",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        });
    });
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<NextState<GameState>>) {
    commands.spawn(Camera2dBundle::default());
    state.set(GameState::Running);
    commands.spawn(
        TextBundle::from_section(
            "WASD to Move around, Q to Melee, E for Ranged, T for AoE, F to Dash",
            TextStyle {
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        }),
    );
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ..Default::default()
    })
    .with_children(|parent| {
        // Health and Score container
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column, // Stack vertically
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(50.0), Val::Px(0.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // Health Text
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Health: 500",
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                ),
                ..Default::default()
            })
            .insert(HealthText);

            // Score Text
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Score: 0",
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                ),
                style: Style {
                    //margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(100.0), Val::Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ScoreText);
        });

        // Ability boxes container at the bottom
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0), 
                height: Val::Px(60.0), // 60px high for the ability boxes
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0), // Position at the bottom of the screen
                justify_content: JustifyContent::SpaceAround, // Evenly space ability boxes
                align_items: AlignItems::Center, // Center the boxes vertically within the container
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            let abilities = [
                Ability::Attack,
                Ability::Ranged,
                Ability::Dash,
                Ability::Aoe,
            ];

            for ability in abilities.iter() {
                let ability_name = match ability {
                    Ability::Attack => "Attack",
                    Ability::Ranged => "Ranged",
                    Ability::Dash => "Dash",
                    Ability::Aoe => "Bladestorm",
                };

                parent.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(20.0),
                        height: Val::Px(50.0), // 50px height for each ability box
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::srgba(0.9, 0.9, 0.9, 0.5).into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("{}: {:.1}s", ability_name, 0.0), // Ability name and placeholder cooldown
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 30.0,
                                color: Color::BLACK,
                            },
                        ),
                        ..Default::default()
                    })
                    .insert(CooldownUi);
                });
            }
        });
    });

    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        dash: asset_server.load(LINE_SPRITE),
        map: asset_server.load(MAP_SPIRITE),
    };

    let enemy_count = EnemyCount(0);

    let mouse_coords = MouseCoords {
        x: 0.,
        y: 0.,
    };

    commands.spawn((
            SpriteBundle {
                texture: game_textures.map.clone(),
                transform: Transform { 
                    //translation: Vec3::new(0., SPRITE_SIZE.1 / 2. + 5., 10.),
                    scale: Vec3::new(4., 4., 0.),
                    ..Default::default()
                },
                ..Default::default()
            },
    ));

    commands.insert_resource(game_textures);
    commands.insert_resource(enemy_count);
    commands.insert_resource(Score::new());
    commands.insert_resource(mouse_coords);
    commands.insert_resource(Points::default());
}

