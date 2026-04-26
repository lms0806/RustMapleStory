use bevy::prelude::*;
use rand::RngExt;

const PLAYER_SIZE: Vec2 = Vec2::new(40.0, 60.0);
const MONSTER_SIZE: Vec2 = Vec2::new(40.0, 50.0);
const PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
const GRAVITY: f32 = 1_200.0;
const PLAYER_SPEED: f32 = 260.0;
const JUMP_FORCE: f32 = 520.0;
const MONSTER_SPEED: f32 = 120.0;
const PROJECTILE_SPEED: f32 = 520.0;
const MONSTER_RESPAWN_MIN_X: f32 = -700.0;
const MONSTER_RESPAWN_MAX_X: f32 = 700.0;
const MONSTER_RESPAWN_Y: f32 = -180.0;
const MONSTER_RESPAWN_SAFE_DISTANCE_FROM_PLAYER: f32 = 220.0;
const MONSTER_RESPAWN_MAX_TRIES: usize = 20;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.11, 0.18)))
        .insert_resource(PlayerFacing(1.0))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust MapleStory Prototype".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_input_system,
                jump_system,
                skill_system,
                gravity_system,
                monster_ai_system,
                movement_system,
                collision_system,
                monster_collision_system,
                projectile_hit_system,
                projectile_cleanup_system,
                camera_follow_system,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Monster;

#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct OnGround(bool);

#[derive(Component)]
struct Collider {
    size: Vec2,
}

#[derive(Component)]
struct Solid;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct Direction(f32);

#[derive(Resource)]
struct PlayerFacing(f32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Ground
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.22, 0.3), Vec2::new(2_000.0, 80.0)),
        Transform::from_xyz(0.0, -260.0, 0.0),
        Collider {
            size: Vec2::new(2_000.0, 80.0),
        },
        Solid,
    ));

    // Platform
    commands.spawn((
        Sprite::from_color(Color::srgb(0.25, 0.3, 0.4), Vec2::new(240.0, 30.0)),
        Transform::from_xyz(220.0, -120.0, 0.0),
        Collider {
            size: Vec2::new(240.0, 30.0),
        },
        Solid,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.75, 0.35), PLAYER_SIZE),
        Transform::from_xyz(-300.0, -180.0, 10.0),
        Player,
        Velocity::default(),
        OnGround(false),
        Collider { size: PLAYER_SIZE },
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.85, 0.25, 0.3), MONSTER_SIZE),
        Transform::from_xyz(250.0, -180.0, 10.0),
        Monster,
        Velocity::default(),
        OnGround(false),
        Collider { size: MONSTER_SIZE },
    ));
}

fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut facing: ResMut<PlayerFacing>,
    mut player_query: Query<&mut Velocity, With<Player>>,
) {
    let Ok(mut vel) = player_query.single_mut() else {
        return;
    };

    let mut axis = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        axis -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        axis += 1.0;
    }

    vel.x = axis * PLAYER_SPEED;
    if axis.abs() > 0.0 {
        facing.0 = axis.signum();
    }
}

fn jump_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &mut OnGround), With<Player>>,
) {
    let Ok((mut vel, mut on_ground)) = player_query.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Space) && on_ground.0 {
        vel.y = JUMP_FORCE;
        on_ground.0 = false;
    }
}

fn gravity_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, Option<&Player>), Without<Projectile>>,
) {
    let dt = time.delta_secs();
    for (mut vel, is_player) in &mut query {
        vel.y -= GRAVITY * dt;
        if is_player.is_some() && vel.y < -1_000.0 {
            vel.y = -1_000.0;
        }
    }
}

fn movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * dt;
        transform.translation.y += vel.y * dt;
    }
}

fn collision_system(
    solids: Query<(&Transform, &Collider), (With<Solid>, Without<Player>)>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &Collider, &mut OnGround),
        (With<Player>, Without<Solid>),
    >,
) {
    let Ok((mut player_transform, mut player_vel, player_collider, mut on_ground)) =
        player_query.single_mut()
    else {
        return;
    };

    on_ground.0 = false;
    for (solid_transform, solid_collider) in &solids {
        if let Some(correction) = resolve_aabb(
            player_transform.translation.truncate(),
            player_collider.size,
            solid_transform.translation.truncate(),
            solid_collider.size,
        ) {
            player_transform.translation.x += correction.x;
            player_transform.translation.y += correction.y;

            if correction.y > 0.0 {
                player_vel.y = 0.0;
                on_ground.0 = true;
            } else if correction.y < 0.0 {
                player_vel.y = 0.0;
            }

            if correction.x.abs() > 0.0 {
                player_vel.x = 0.0;
            }
        }
    }
}

fn monster_collision_system(
    solids: Query<(&Transform, &Collider), (With<Solid>, Without<Monster>)>,
    mut monster_query: Query<
        (&mut Transform, &mut Velocity, &Collider, &mut OnGround),
        (With<Monster>, Without<Solid>),
    >,
) {
    for (mut monster_transform, mut monster_vel, monster_collider, mut on_ground) in &mut monster_query {
        on_ground.0 = false;

        for (solid_transform, solid_collider) in &solids {
            if let Some(correction) = resolve_aabb(
                monster_transform.translation.truncate(),
                monster_collider.size,
                solid_transform.translation.truncate(),
                solid_collider.size,
            ) {
                monster_transform.translation.x += correction.x;
                monster_transform.translation.y += correction.y;

                if correction.y > 0.0 {
                    monster_vel.y = 0.0;
                    on_ground.0 = true;
                } else if correction.y < 0.0 {
                    monster_vel.y = 0.0;
                }

                if correction.x.abs() > 0.0 {
                    monster_vel.x = 0.0;
                }
            }
        }
    }
}

fn monster_ai_system(
    player_query: Query<&Transform, (With<Player>, Without<Monster>)>,
    mut monster_query: Query<(&Transform, &mut Velocity), With<Monster>>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let player_x = player_tf.translation.x;

    for (monster_tf, mut vel) in &mut monster_query {
        let direction = (player_x - monster_tf.translation.x).signum();
        vel.x = direction * MONSTER_SPEED;
    }
}

fn skill_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    facing: Res<PlayerFacing>,
    player_query: Query<&Transform, With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::ControlLeft) && !keyboard.just_pressed(KeyCode::ControlRight)
    {
        return;
    }
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    let dir = if facing.0.abs() < 0.1 {
        1.0
    } else {
        facing.0.signum()
    };

    commands.spawn((
        Sprite::from_color(Color::BLACK, PROJECTILE_SIZE),
        Transform::from_xyz(
            player_tf.translation.x + dir * (PLAYER_SIZE.x * 0.6),
            player_tf.translation.y + 10.0,
            20.0,
        ),
        Projectile,
        Direction(dir),
        Velocity(Vec2::new(dir * PROJECTILE_SPEED, 0.0)),
        Collider {
            size: PROJECTILE_SIZE,
        },
    ));
}

fn projectile_hit_system(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Monster>)>,
    projectile_query: Query<(Entity, &Transform, &Collider), (With<Projectile>, Without<Monster>)>,
    mut monster_query: Query<
        (Entity, &mut Transform, &mut Velocity, &mut OnGround, &Collider),
        (With<Monster>, Without<Projectile>),
    >,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let player_x = player_tf.translation.x;

    for (projectile_entity, projectile_tf, projectile_col) in &projectile_query {
        for (_monster_entity, mut monster_tf, mut monster_vel, mut monster_on_ground, monster_col) in
            &mut monster_query
        {
            let hit = intersects_aabb(
                projectile_tf.translation.truncate(),
                projectile_col.size,
                monster_tf.translation.truncate(),
                monster_col.size,
            );

            if hit {
                commands.entity(projectile_entity).despawn();

                let mut rng = rand::rng();
                let mut random_x = monster_tf.translation.x;
                for _ in 0..MONSTER_RESPAWN_MAX_TRIES {
                    let candidate = rng.random_range(MONSTER_RESPAWN_MIN_X..=MONSTER_RESPAWN_MAX_X);
                    if (candidate - player_x).abs() >= MONSTER_RESPAWN_SAFE_DISTANCE_FROM_PLAYER {
                        random_x = candidate;
                        break;
                    }
                }
                monster_tf.translation.x = random_x;
                monster_tf.translation.y = MONSTER_RESPAWN_Y;
                monster_vel.0 = Vec2::ZERO;
                monster_on_ground.0 = false;
                break;
            }
        }
    }
}

fn projectile_cleanup_system(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Direction), With<Projectile>>,
) {
    for (entity, transform, direction) in &projectile_query {
        if transform.translation.x.abs() > 2_000.0 || direction.0 == 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let Ok(mut camera_tf) = camera_query.single_mut() else {
        return;
    };

    camera_tf.translation.x = player_tf.translation.x;
    camera_tf.translation.y = player_tf.translation.y + 40.0;
}

fn intersects_aabb(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let delta = a_pos - b_pos;
    let px = (a_size.x + b_size.x) * 0.5 - delta.x.abs();
    let py = (a_size.y + b_size.y) * 0.5 - delta.y.abs();
    px > 0.0 && py > 0.0
}

fn resolve_aabb(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> Option<Vec2> {
    let delta = a_pos - b_pos;
    let px = (a_size.x + b_size.x) * 0.5 - delta.x.abs();
    let py = (a_size.y + b_size.y) * 0.5 - delta.y.abs();

    if px <= 0.0 || py <= 0.0 {
        return None;
    }

    if px < py {
        let sx = if delta.x < 0.0 { -1.0 } else { 1.0 };
        Some(Vec2::new(px * sx, 0.0))
    } else {
        let sy = if delta.y < 0.0 { -1.0 } else { 1.0 };
        Some(Vec2::new(0.0, py * sy))
    }
}
