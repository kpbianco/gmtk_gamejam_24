use bevy::audio::{PlaybackMode, Volume};
use bevy::input::keyboard::Key;
use bevy::input::mouse::{self, MouseMotion};
use bevy::{prelude::*, time};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::transform::commands;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use bevy::ui::{AlignItems, JustifyContent, Val, UiRect, Style};
use crate::components::{wallpaper, Ability, Bigfoot, BigfootState, Collider, CooldownUi, Cooldowns, GameOverUI, GameTimer, GameTimerText, GameUI, Health, HealthText, Invulnerability, Lifetime, Line, Map, MapGrid, MenuUI, MousePosition, MovementSpeed, PauseMenu, Player, PointMarker, Points, QuitButton, Resettable, RestartButton, Score, ScoreText, StartButton, Velocity};
use crate::events::CollisionEvent;
use crate::player::{self, player_spawn_system};
use crate::{EnemySpawnRate, GameState, MAP_SPIRITE};

use rand::Rng;
use std::f32::consts::PI;
use std::time::Duration;
use crate::{GameTextures, MouseCoords, ENEMY_SPRITE, LINE_SPRITE, PLAYER_SPRITE};
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

pub fn clean_dead(

    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity_id, entity_health) in query.iter() {
        if entity_health.hp <=0 {
            commands.entity(entity_id).despawn();
        }
    }
}

//pub fn enemy_killed(score: &mut ResMut<Score>, mut player: &mut Player, cooldowns_query: &mut Query<&mut Cooldowns>,) {
//    score.increment();
//    player.heal(1);
//    println!("Score: {}", score.get_enemies_killed());
//     // Apply cooldown reduction to all abilities
//     // Apply cooldown reduction to all abilities
//     for mut cooldowns in cooldowns_query.iter_mut() {
//        for timer in cooldowns.cooldowns.values_mut() {
//            // Calculate the new elapsed time, ensuring it doesn't go below zero
//            let elapsed_time = timer.elapsed_secs() + 0.05;
//
//            // Manually set the timer's tick to the new elapsed time
//            timer.set_elapsed(Duration::from_secs_f32(elapsed_time.min(timer.duration().as_secs_f32())));
//        }
//    }
//}

pub fn spawn_bigfoot(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;
        println!("Bigfoot spawned");

        commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("foot.png"), // Assuming a texture is available
                    transform: Transform {
                        translation: Vec3::new(100., player_position.y, 1.0),
                        //translation: Vec3::new(player_position.x, player_position.y, 0.0),
                        scale: Vec3::new(0.7, 0.7, 1.0), // Adjusted scale for a 250 radius
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Bigfoot {
                    timer: Timer::from_seconds(2.5, TimerMode::Once),
                    state: BigfootState::Invulnerable,
                    health: 5,
                    x: player_position.x,  // Store the initial position
                    y: player_position.y,  // Store the initial position
                    airTexture: asset_server.load("foot.png"),
                    groundTexture: asset_server.load("foot_ground.png")
                },
                Collider::new(Vec2::new(256., 256.)),
                ));
    }
}


pub fn update_bigfoot(
    mut query: Query<(Entity, &mut Bigfoot, &mut Sprite, &mut Transform, &mut Handle<Image>), Without<Player>>,
    mut player_query: Query<(&mut Transform, Option<&mut Invulnerability>), With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
) {
    for (entity, mut bigfoot, mut sprite, mut transform, mut texture) in query.iter_mut() {
        // Update Bigfoot's timer
        bigfoot.timer.tick(time.delta());

        if bigfoot.timer.just_finished() {
            match bigfoot.state {
                BigfootState::Invulnerable => {
                    // Switch to the stomp phase
                    bigfoot.state = BigfootState::Solid;

                    // Make Bigfoot fully opaque and solid
                    sprite.color.set_alpha(1.0);

                    // Set the timer for the stomp phase
                    bigfoot.timer = Timer::from_seconds(5.0, TimerMode::Once);

                    // Change the texture based on the state
                    cycle_texture(&mut texture, &bigfoot);
                    stomp_sound(&asset_server, &mut commands);

                    if let Ok((mut player_transform, mut invulnerability_option)) = player_query.get_single_mut() {
                        let player_position = Vec3 { x: player_transform.translation.x, y: player_transform.translation.y, z: 1.0 };
                        let bigfoot_position = Vec3 { x: bigfoot.x, y: bigfoot.y, z: 1.0 };

                        // If the player is within the 250 radius, apply damage
                        let distance = player_position.distance(bigfoot_position);
                        if distance <= 175.0 {
                            //if let Some(ref mut invulnerability) = invulnerability_option {
                            //    player.take_damage(
                            //        500, // Damage amount
                            //        entity,
                            //        &mut commands,
                            //        Some(invulnerability), // Pass the mutable reference
                            //        &mut state,
                            //        &asset_server
                            //    );
                            //} else {
                            //    player.take_damage(
                            //        500, // Damage amount
                            //        entity,
                            //        &mut commands,
                            //        None, 
                            //        &mut state,
                            //        &asset_server
                            //    );
                            //}
                        }
                    }
                }
                BigfootState::Solid => {
                    // Bigfoot has finished stomping, reset its state and move it to the player's position

                    // Get the player's position
                    if let Ok((player_transform, _invulnerability_option)) = player_query.get_single() {
                        // Move Bigfoot to the player_velocity's position
                        bigfoot.x = player_transform.translation.x;
                        bigfoot.y = player_transform.translation.y;

                        // Update the transform of Bigfoot to match the new position
                        transform.translation.x = bigfoot.x;
                        transform.translation.y = bigfoot.y;

                        // Reset Bigfoot's state to Invulnerable and restart the timer
                        bigfoot.state = BigfootState::Invulnerable;
                        bigfoot.timer = Timer::from_seconds(2.5, TimerMode::Once);

                        // Make Bigfoot semi-transparent again
                        sprite.color.set_alpha(0.5);
                        cycle_texture(&mut texture, &bigfoot);
                    }
                }
                BigfootState::Cleanup => todo!(),
            }
        } else if bigfoot.state == BigfootState::Invulnerable {
            // While Bigfoot is invulnerable, make it semi-transparent
            sprite.color.set_alpha(0.5);
        }
    }
}

fn cycle_texture(
    texture: &mut Handle<Image>,
    bigfoot: &Bigfoot,
) {
    if *texture == bigfoot.airTexture {
        *texture = bigfoot.groundTexture.clone();
    } else {
        *texture = bigfoot.airTexture.clone();
    }
}

//pub fn update_player_position(
//    mut player_query: Query<(&mut Velocity, &Transform), With<Player>>,
//) {
//    for (mut player, transform) in player_query.iter_mut() {
//        player.x = transform.translation.x;
//        player.y = transform.translation.y;
//    }
//}

pub fn update_bigfoot_position(
    mut bigfoot_query: Query<(&mut Bigfoot, &Transform)>,
) {
    for (mut bigfoot, transform) in bigfoot_query.iter_mut() {
        bigfoot.x = transform.translation.x;
        bigfoot.y = transform.translation.y;
    }
}


pub fn setup_menu(mut commands:  Commands, asset_server:  Res<AssetServer>, player_query: Query<&Transform, With<Player>>,) {

    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;

        commands.spawn(
            SpriteBundle {
                texture: asset_server.load("./wallpaper.png"), // Assuming a texture is available
                transform: Transform {
                    translation: Vec3::new(player_position.x, player_position.y, 3.0),
                    //translation: Vec3::new(player_position.x, player_position.y, 0.0),
                    //scale: Vec3::new(0.7, 0.7, 1.0), // Adjusted scale for a 250 radius
                    ..Default::default()
                },
                ..Default::default()
            },)
        .insert(wallpaper);

    }

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,  // Stack elements vertically
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(GameOverUI)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                          "Gashadokuro Escape",
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 120.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            })
            .insert(wallpaper);

            parent.spawn(TextBundle {
                text: Text::from_section(
                          "Defeat the skeleton without being hit to win",
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 60.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            })
            .insert(wallpaper);;

            parent.spawn(TextBundle {
                text: Text::from_section(
                          "WASD to Move around, Q to Melee, E for Ranged, T for AoE, F to Dash",
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 30.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            })
            .insert(wallpaper);;
            });
    // Root node

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        //background_color: Color::srgba(0.15, 0.15, 0.15, 1.0).into(),
        ..Default::default()

    }).insert(MenuUI)
    //.insert(background_handle)
    .with_children(|parent| {
        // Start button
        parent.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(200.0), 
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::srgba(0.25, 0.25, 0.75, 1.0).into(),
            ..Default::default()
        })
        .insert(StartButton)
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                              "Start",
                              TextStyle {
                                  font: asset_server.load("FiraSans-Bold.ttf"),
                                  font_size: 40.0,
                                  color: Color::WHITE,
                              },
                          ),
                          ..Default::default()
                });
            });

        // Quit button
        parent.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(200.0), 
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::srgba(0.75, 0.25, 0.25, 1.0).into(),
            ..Default::default()
        })
        .insert(QuitButton)
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                              "Quit",
                              TextStyle {
                                  font: asset_server.load("FiraSans-Bold.ttf"),
                                  font_size: 40.0,
                                  color: Color::WHITE,
                              },
                          ),
                          ..Default::default()
                });
            });
    });
}

pub fn setup_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
    game_timer: Res<GameTimer>,
) {
    let background_handle: Handle<Image> = asset_server.load("./wallpaper.png");

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,  // Stack elements vertically
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(GameOverUI)
        .insert(background_handle)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                          format!("You fell to Gashadokuru!"),
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 100.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            });
            parent.spawn(TextBundle {
                text: Text::from_section(
                          format!("Final Score: {}", score.get_enemies_killed()),
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 40.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            });

            parent.spawn(TextBundle {
                text: Text::from_section(
                          format!("Time Survived: {:.1} seconds", game_timer.0),
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 40.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            });

            // Add a spacing node between the text and the buttons
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(50.0), // Spacing
                    ..Default::default()
                },
                ..Default::default()
            });

            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(200.0), 
                    height: Val::Px(65.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::srgba(0.25, 0.75, 0.25, 1.0).into(),
                ..Default::default()
            })

            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                              format!("Final Score: {}", score.get_enemies_killed()),
                              TextStyle {
                                  font: asset_server.load("FiraSans-Bold.ttf"),
                                  font_size: 40.0,
                                  color: Color::WHITE,
                              },
                          ),
                          ..Default::default()
                });
            }).insert(RestartButton);
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(200.0), 
                    height: Val::Px(65.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::srgba(0.75, 0.25, 0.25, 0.5).into(),
                ..Default::default()
            })
            .insert(QuitButton)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                                  format!("Time Survived: {:.1} seconds", game_timer.0),
                                  TextStyle {
                                      font: asset_server.load("FiraSans-Bold.ttf"),
                                      font_size: 40.0,
                                      color: Color::WHITE,
                                  },
                              ),
                              ..Default::default()
                    });

                    // Add a spacing node between the text and the buttons
                    parent.spawn(NodeBundle {
                        style: Style {
                            height: Val::Px(50.0), // Spacing
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                    parent.spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0), 
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: Color::srgba(0.25, 0.75, 0.25, 1.0).into(),
                        ..Default::default()
                    })
                    .insert(StartButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                          "Play Again",
                                          TextStyle {
                                              font: asset_server.load("FiraSans-Bold.ttf"),
                                              font_size: 40.0,
                                              color: Color::WHITE,
                                          },
                                      ),
                                      ..Default::default()
                            });
                        });
                    parent.spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0), 
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: Color::srgba(0.75, 0.25, 0.25, 0.5).into(),
                        ..Default::default()
                    })
                    .insert(QuitButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                          "Quit",
                                          TextStyle {
                                              font: asset_server.load("FiraSans-Bold.ttf"),
                                              font_size: 40.0,
                                              color: Color::WHITE,
                                          },
                                      ),
                                      ..Default::default()
                            });
                        });

                    // Other buttons and UI elements
                });
        });
}

pub fn won_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
    game_timer: Res<GameTimer>,
    player_query: Query<&Transform, With<Player>>,
) {
    let background_handle: Handle<Image> = asset_server.load("./victory.png");

    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;
        commands.spawn(
            SpriteBundle {
                texture: asset_server.load("./victory.png"), // Assuming a texture is available
                transform: Transform {
                    translation: Vec3::new(player_position.x, player_position.y, 3.0),
                    //translation: Vec3::new(player_position.x, player_position.y, 0.0),
                    //scale: Vec3::new(0.7, 0.7, 1.0), // Adjusted scale for a 250 radius
                    ..Default::default()
                },
                ..Default::default()
            },)
        .insert(wallpaper);
    }
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,  // Stack elements vertically
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(GameOverUI)
        .insert(background_handle)

        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                          format!("Gashadokuru Slain!"),
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 100.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            });
            parent.spawn(TextBundle {
                text: Text::from_section(
                          format!("Final Score: {}", score.get_enemies_killed()),
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 40.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            });

            parent.spawn(TextBundle {
                text: Text::from_section(
                          format!("Time Survived: {:.1} seconds", game_timer.0),
                          TextStyle {
                              font: asset_server.load("FiraSans-Bold.ttf"),
                              font_size: 40.0,
                              color: Color::WHITE,
                          },
                      ),
                      ..Default::default()
            });

            // Add a spacing node between the text and the buttons
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(50.0), // Spacing
                    ..Default::default()
                },
                ..Default::default()
            });

            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(200.0), 
                    height: Val::Px(65.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::srgba(0.25, 0.75, 0.25, 1.0).into(),
                ..Default::default()
            })

            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                              "Play Again",
                              TextStyle {
                                  font: asset_server.load("FiraSans-Bold.ttf"),
                                  font_size: 40.0,
                                  color: Color::WHITE,
                              },
                          ),
                          ..Default::default()
                });
            }).insert(RestartButton);
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(200.0), 
                    height: Val::Px(65.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::srgba(0.75, 0.25, 0.25, 0.5).into(),
                ..Default::default()
            })
            .insert(QuitButton)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                                  "Quit",
                                  TextStyle {
                                      font: asset_server.load("FiraSans-Bold.ttf"),
                                      font_size: 40.0,
                                      color: Color::WHITE,
                                  },
                              ),
                              ..Default::default()
                    });
                });

            // Other buttons and UI elements
        });
}

pub fn check_won_game(
    mut commands:  Commands,
    query: Query<Entity, With<MenuUI>>,
    state: ResMut<State<GameState>>,
    mut asset_server:   Res<AssetServer>,
    score: Res<Score>,
    game_timer: Res<GameTimer>,
    player_query: Query<&Transform, With<Player>>,
) {
    if *state.get() == GameState::Won {
        println!("won");
        won_game(commands, asset_server, score, game_timer, player_query);
    }
}


pub fn spawn_menu(
    mut commands:  Commands,
    query: Query<Entity, With<MenuUI>>,
    state: ResMut<State<GameState>>,
    mut asset_server:   Res<AssetServer>,
) {
    if *state.get() == GameState::Running || *state.get() == GameState::Paused{
        game_menus(&mut commands,  &mut asset_server);
    }
}

pub fn despawn_menu(
    mut commands: Commands,
    query: Query<Entity, With<MenuUI>>,
    mut state: ResMut<State<GameState>>,
) {
    if *state.get() == GameState::Running {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn kill_wallpaper(
    mut commands: Commands,
    query: Query<Entity, With<wallpaper>>,
    mut state: ResMut<State<GameState>>,
) {

    for entity in query.iter() {
        println!("herre");
        commands.entity(entity).despawn_recursive();
    }
}

pub fn kill_game_over_ui(
    mut commands: Commands,
    query: Query<Entity, With<GameOverUI>>,
    mut state: ResMut<State<GameState>>,
) {

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn kill_game_ui(
    mut commands: Commands,
    query: Query<Entity, With<GameUI>>,
    mut state: ResMut<State<GameState>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn menu_action_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &StartButton), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut asset_server:  Res<AssetServer>,
) {
    for (interaction, mut color, _start_button) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                state.set(GameState::Reset);
                menu_sound(&asset_server, &mut commands);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                menu_sound(&asset_server, &mut commands);
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.75));
            }
        }
    }
}

pub fn restart_action_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &RestartButton), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut asset_server:  Res<AssetServer>,
) {
    for (interaction, mut color, _start_button) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                state.set(GameState::Reset);
                menu_sound(&asset_server, &mut commands);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                menu_sound(&asset_server, &mut commands);
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.75));
            }
        }
    }
}

pub fn quit_action_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &QuitButton), (Changed<Interaction>, With<Button>)>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut asset_server:  Res<AssetServer>,
) {
    for (interaction, mut color, _quit_button) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                menu_sound(&asset_server, &mut commands);
                exit.send(AppExit::Success);
            }
            Interaction::Hovered => {
                menu_sound(&asset_server, &mut commands);
                *color = Color::srgb(0.75, 0.35, 0.35).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.75, 0.25, 0.25).into();
            }
        }
    }
}

pub fn handle_escape_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut curr_state: ResMut<State<GameState>>,
    mut commands: Commands,
    mut asset_server:  Res<AssetServer>,
    query: Query<Entity, With<MenuUI>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        menu_sound(&asset_server, &mut commands);
        println!("gaming");
        if *curr_state.get() == GameState::Running {
            state.set(GameState::Paused);
        } else if *curr_state.get() == GameState::Paused {
            state.set(GameState::Running);
        }
    }else if keyboard_input.just_pressed(KeyCode::KeyB) {
        if *curr_state.get() == GameState::Paused {
            state.set(GameState::Menu);
            //despawn_menu(commands, query, asset_server.as_must);
            //setup_menu(commands, asset_server);
        }
    }  
}


//pub fn flicker_system(
//    time: Res<Time>,
//    mut query: Query<(&mut Sprite, &mut Invulnerability), With<Player>>,
//) {
//    for (mut sprite, mut invulnerability) in query.iter_mut() {
//        // Update the timer for invulnerability
//        invulnerability.timer.tick(time.delta());
//
//        // If the player is invulnerable, adjust the alpha value to create a flicker effect
//        if invulnerability.is_active() {
//            // Flicker by adjusting alpha value between 0.2 and 1.0
//            let flicker_phase = f32::sin(15.0); //maybe fixed
//            let new_alpha = 0.5 * flicker_phase.abs();
//
//            // Directly set the alpha using set_alpha
//            sprite.color.set_alpha(new_alpha);
//        } else {
//            // Ensure the player is fully visible when not invulnerable
//            sprite.color.set_alpha(0.99);
//        }
//    }
//}

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
    player_query: Query<&Health, With<Player>>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    mut text_query: Query<(&mut Text, Option<&HealthText>, Option<&ScoreText>, Option<&GameTimerText>)>,
) {
    if let Ok(player_health) = player_query.get_single() {
        for (mut text, health_text, score_text, timer_text) in text_query.iter_mut() {
            if health_text.is_some() {
                text.sections[0].value = format!("Health: {}", player_health.hp);
            } else if score_text.is_some() {
                text.sections[0].value = format!("Score: {}", score.get_enemies_killed());
            }else if timer_text.is_some() {
                text.sections[0].value = format!("Time: {}", f32::trunc(timer.0 * 100.0)/ 100.)
            }
        }
    }
}

const MAP_WIDTH: f32 = 2672.0*4.0;
const MAP_HEIGHT: f32 = 1312.0*4.0;
const MAP_SPAWN_THRESHOLD: f32 = 500.0; // Adjust as necessary


pub fn check_and_spawn_map(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut map_query: Query<(Entity, &Transform), With<Map>>,
    mut map_grid: ResMut<MapGrid>,
    game_textures: Res<GameTextures>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;

        // Calculate the player's grid position
        let player_grid_x = (player_pos.x / MAP_WIDTH).round() as i32;
        let player_grid_y = (player_pos.y / MAP_HEIGHT).round() as i32;

        // Check the surrounding 8 grid positions and spawn maps if necessary
        for dx in -1..=1 {
            for dy in -1..=1 {
                let grid_x = player_grid_x + dx;
                let grid_y = player_grid_y + dy;

                if !map_grid.positions.contains(&(grid_x, grid_y)) {
                    // Spawn a new map at this grid position
                    commands.spawn((
                            SpriteBundle {
                                texture: game_textures.map.clone(),
                                transform: Transform {
                                    translation: Vec3::new(grid_x as f32 * MAP_WIDTH, grid_y as f32 * MAP_HEIGHT, 0.0),
                                    scale: Vec3::new(4.0, 4.0, 0.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            Map,
                    ));
                    // Add the new map's position to the grid
                    map_grid.positions.insert((grid_x, grid_y));
                }
            }
        }

        // Now, clean up maps that are not adjacent to the player's position
        for (entity, map_transform) in map_query.iter_mut() {
            let map_pos = map_transform.translation;
            let map_grid_x = (map_pos.x / MAP_WIDTH).round() as i32;
            let map_grid_y = (map_pos.y / MAP_HEIGHT).round() as i32;

            // Check if the map is within a 3x3 grid around the player
            if (map_grid_x - player_grid_x).abs() > 1 || (map_grid_y - player_grid_y).abs() > 1 {
                // Despawn maps that are outside this grid
                commands.entity(entity).despawn();
                map_grid.positions.remove(&(map_grid_x, map_grid_y));
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
                          "Game Paused\nPress Esc to Resume\n\n\n 'B' To Go To Main Menu",
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

pub fn menu_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/select.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(1.3),
            ..Default::default()
        }
    });
}

pub fn death_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/death.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.3),
            ..Default::default()
        }
    });
}

pub fn play_empty_swing(
    asset_server: Res<AssetServer>,
    mut commands: &mut Commands
) {
    let sound1 = "sfx/swing1.ogg";
    let sound2 = "sfx/swing2.ogg";
    let sound3 = "sfx/swing3.ogg";

    // Collect the sounds into a vector
    let sounds = vec![sound1, sound2, sound3];

    // Generate a random index to pick a sound
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..sounds.len());

    // Select the sound based on the random index
    let selected_sound = sounds[random_index];
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load(selected_sound),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(1.0),
            ..Default::default()
        }
    });
}

pub fn play_hit_swing(
    asset_server: & Res<AssetServer>,
    commands: &mut Commands
) {
    let sound1 = "sfx/hit1.ogg";
    let sound2 = "sfx/hit2.ogg";
    let sound3 = "sfx/hit3.ogg";

    // Collect the sounds into a vector
    let sounds = vec![sound1, sound2, sound3];

    // Generate a random index to pick a sound
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..sounds.len());

    // Select the sound based on the random index
    let selected_sound = sounds[random_index];
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load(selected_sound),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(1.0),
            ..Default::default()
        }
    });
}

pub fn bone_hit(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/bone.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.15),
            ..Default::default()
        }
    });
}

pub fn dash_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/dash.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(2.75),
            ..Default::default()
        }
    });
}

pub fn reset_game(
    mut commands: Commands,
    mut player_query: Query<&mut Player>,
    mut bigfoot_query: Query<&mut Bigfoot>,
    enemy_query: Query<Entity, (With<Resettable>, Without<Player>)>,
    mut score: ResMut<Score>,
    mut game_timer: ResMut<GameTimer>,
    mut state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    player_transform: Query<&Transform, With<Player>>, 
    asset_server: Res<AssetServer>,
    mut cooldowns_query: Query<&mut Cooldowns>,
) {
    // Only proceed if the game state is Reset
    if *state.get() == GameState::Reset {
        // Reset player health and position
        //if let Ok(mut player) = player_query.get_single_mut() {
        //    player.health = 500;
        //    player.x = 0.0;
        //    player.y = 0.0;
        //}

        // Despawn all enemies with the Spawned tag
        for enemy_entity in enemy_query.iter() {
            commands.entity(enemy_entity).despawn_recursive();
        }

        if let Ok(mut cooldowns) = cooldowns_query.get_single_mut() {
            cooldowns.reset_all();
        }

        // Spawn Bigfoot
        spawn_bigfoot(commands, player_transform, asset_server);

        // Reset score
        score.reset();

        // Reset game timer
        game_timer.0 = 0.0;

        // Transition back to the Running state
        next_state.set(GameState::Running);
    }
}

pub fn aoe_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/aoe.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.8),
            ..Default::default()
        }
    });
}
pub fn update_timer(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<GameTimerText>>,
) {
    // Accumulate time
    timer.0 += time.delta_seconds();

    // Update the text with the accumulated time
    // for mut text in query.iter_mut() {
    //     text.sections[0].value = format!("Time: {:.1}", timer.0);
    // }
}

pub fn stomp_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/stomp.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.6),
            ..Default::default()
        }
    });
}

pub fn ranged_sound(
    asset_server: &mut Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/ranged.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.6),
            ..Default::default()
        }
    });
}

pub fn game_menus(    commands: &mut Commands,
    asset_server:  &mut Res<AssetServer>,) {
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
    ).insert(GameUI);

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(GameUI)
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
            .insert(GameUI)
                .with_children(|parent| {
                    // Health Text
                    parent.spawn(TextBundle {
                        text: Text::from_section(

                                  "Health: 100",
                                  TextStyle {
                                      font: asset_server.load("FiraSans-Bold.ttf"),
                                      font_size: 40.0,
                                      color: Color::WHITE,
                                  },
                              ),
                              ..Default::default()
                    })
                    .insert(Resettable)
                        .insert(HealthText)
                        .insert(GameUI);

                    parent.spawn(TextBundle {
                        text: Text::from_section(
                                  "Time: 0.0",
                                  TextStyle {
                                      font: asset_server.load("FiraSans-Bold.ttf"),
                                      font_size: 40.0,
                                      color: Color::WHITE,
                                  },
                              ),
                              ..Default::default()
                    })
                    .insert(Resettable)

                        .insert(GameTimerText)
                        .insert(GameUI);

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
                    .insert(Resettable)
                        .insert(ScoreText)
                        .insert(GameUI);
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
                        .insert(CooldownUi)
                            .insert(Resettable)
                            .insert(GameUI);
                        });
                }
            });
        });
}



pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<NextState<GameState>>) {
    commands.spawn(Camera2dBundle::default());
    state.set(GameState::Menu);


    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        line: asset_server.load(LINE_SPRITE),
        map: asset_server.load(MAP_SPIRITE),
    };

    let enemy_count = EnemySpawnRate(2.0);

    let mouse_coords = MouseCoords {
        x: 0.,
        y: 0.,
    };

    // Create an entity dedicated to playing our background music
    commands.spawn(AudioBundle {
        source: asset_server.load("./beats/back.ogg"),
        settings: PlaybackSettings::LOOP,
    });
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
    )).insert(Map)
        .insert(Resettable);

    commands.insert_resource(game_textures);
    commands.insert_resource(enemy_count);
    commands.insert_resource(mouse_coords);
    commands.insert_resource(Points::default());
}

pub fn cleanup_game(mut commands:   Commands, 
    mut asset_server:   Res<AssetServer>, 
    player_query: Query<&Transform, With<Player>>, 
    mut score: ResMut<Score>, 
    mut points: ResMut<Points>, 
    mut state: ResMut<NextState<GameState>>,
    mut game_textures: Res<GameTextures>, 
    query: Query<Entity, (With<Resettable>)>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    score.reset();
    *points = Points::default();

    //game_menus( &mut commands, &mut asset_server);
    spawn_bigfoot(commands, player_query, asset_server);
    // player_spawn_system(commands, game_textures);

}
