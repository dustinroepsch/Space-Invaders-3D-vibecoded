use bevy::prelude::*;
use rand::Rng;

use crate::components::*;

pub struct MysteryPlugin;

impl Plugin for MysteryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MysteryShipTimer>()
            .add_message::<MysteryShipKilledEvent>()
            .add_systems(
                Update,
                (spawn_mystery_ship, despawn_mystery_ship, handle_mystery_ship_killed)
                    .run_if(in_state(GameState::Playing)),
            )
            // Popup ticking runs in all states so the animation completes even if the
            // wave clears or game ends in the same frame as the kill.
            .add_systems(Update, tick_score_popups);
    }
}

fn spawn_mystery_ship(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MysteryShipTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<(), With<MysteryShip>>,
) {
    timer.timer.tick(time.delta());
    if !timer.timer.just_finished() || !existing.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    // Vary direction each appearance.
    let from_left = rng.gen_bool(0.5);
    let start_x = if from_left {
        -ARENA_HALF_WIDTH - 3.0
    } else {
        ARENA_HALF_WIDTH + 3.0
    };
    let vel_x = if from_left {
        MYSTERY_SHIP_SPEED
    } else {
        -MYSTERY_SHIP_SPEED
    };

    // Point values matching the original game's table.
    let bonus_table = [50u32, 100, 150, 100, 200, 100, 300, 100];
    let bonus = bonus_table[rng.gen_range(0..bonus_table.len())];

    // --- Materials ---
    let hull_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.08, 0.15),
        emissive: LinearRgba::new(10.0, 0.8, 1.5, 1.0),
        metallic: 0.85,
        perceptual_roughness: 0.15,
        ..default()
    });
    let dome_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.85, 1.0),
        emissive: LinearRgba::new(3.0, 4.0, 12.0, 1.0),
        metallic: 0.95,
        perceptual_roughness: 0.05,
        ..default()
    });
    let nacelle_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.4, 0.05),
        emissive: LinearRgba::new(10.0, 3.0, 0.3, 1.0),
        metallic: 0.6,
        perceptual_roughness: 0.3,
        ..default()
    });

    // --- Meshes ---
    // Main flat disk hull
    let hull_mesh = meshes.add(Cuboid::new(2.4, 0.25, 1.0));
    // Narrower mid-section
    let mid_mesh = meshes.add(Cuboid::new(1.6, 0.22, 0.75));
    // Glass dome on top
    let dome_mesh = meshes.add(Cuboid::new(0.85, 0.42, 0.55));
    // Engine nacelles (two small blocks on each side)
    let nacelle_mesh = meshes.add(Cuboid::new(0.28, 0.18, 0.65));

    commands
        .spawn((
            Mesh3d(hull_mesh),
            MeshMaterial3d(hull_mat.clone()),
            Transform::from_xyz(start_x, MYSTERY_SHIP_Y, MYSTERY_SHIP_Z),
            MysteryShip,
            MysteryShipPoints(bonus),
            Velocity(Vec3::new(vel_x, 0.0, 0.0)),
        ))
        .with_children(|p| {
            // Mid-section band
            p.spawn((
                Mesh3d(mid_mesh),
                MeshMaterial3d(hull_mat.clone()),
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
            // Dome
            p.spawn((
                Mesh3d(dome_mesh),
                MeshMaterial3d(dome_mat),
                Transform::from_xyz(0.0, 0.38, 0.0),
            ));
            // Left nacelle
            p.spawn((
                Mesh3d(nacelle_mesh.clone()),
                MeshMaterial3d(nacelle_mat.clone()),
                Transform::from_xyz(-0.9, -0.1, 0.0),
            ));
            // Right nacelle
            p.spawn((
                Mesh3d(nacelle_mesh),
                MeshMaterial3d(nacelle_mat),
                Transform::from_xyz(0.9, -0.1, 0.0),
            ));
        });
}

fn despawn_mystery_ship(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<MysteryShip>>,
) {
    for (entity, transform) in &query {
        if transform.translation.x.abs() > ARENA_HALF_WIDTH + 4.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_mystery_ship_killed(
    mut commands: Commands,
    mut events: MessageReader<MysteryShipKilledEvent>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ui_scale: Res<UiScale>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Val::Px values are in "UI units". With UiScale != 1.0 a UI unit no longer
    // equals a logical pixel, so we divide the viewport-pixel coordinates
    // returned by world_to_viewport by the current scale before storing them.
    let scale = ui_scale.0.max(0.001);

    for event in events.read() {
        // Project the 3-D kill position into screen (viewport) pixels.
        let screen_pos = camera
            .world_to_viewport(camera_transform, event.world_pos)
            .unwrap_or(Vec2::new(400.0, 120.0));

        let top_ui = screen_pos.y / scale;
        let left_ui = (screen_pos.x - 28.0) / scale;

        commands.spawn((
            Text::new(format!("+{}", event.points)),
            TextFont {
                font_size: 34.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.85, 0.15, 1.0)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left_ui),
                top: Val::Px(top_ui),
                ..default()
            },
            ScorePopup {
                timer: Timer::from_seconds(1.5, TimerMode::Once),
                start_top: top_ui,
            },
        ));
    }
}

fn tick_score_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScorePopup, &mut Node, &mut TextColor)>,
) {
    for (entity, mut popup, mut node, mut color) in &mut query {
        popup.timer.tick(time.delta());
        let t = popup.timer.fraction();

        // Float the text upward 70 px over its lifetime.
        node.top = Val::Px(popup.start_top - t * 70.0);

        // Fade out over the second half of the lifetime.
        let alpha = if t < 0.5 { 1.0 } else { 1.0 - (t - 0.5) * 2.0 };
        color.0 = Color::srgba(1.0, 0.85, 0.15, alpha);

        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
