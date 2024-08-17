mod components;
mod systems;
mod events;

use bevy::prelude::*;
use components::*;
use systems::*;
use events::*;

fn main() {
    App::new()
        .insert_resource(Score::new())
        .insert_resource(MousePosition::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                sprite_movement,
                pathfind_towards_player,
                move_entities,
                display_score,
                check_collisions,
                handle_collisions,
                spawn_enemies_over_time,
                camera_follow_player,
                track_cursor_position
            ),
        )
        .add_event::<CollisionEvent>()
        .run();
}
