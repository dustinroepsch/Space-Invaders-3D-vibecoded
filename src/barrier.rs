use bevy::prelude::*;

use crate::components::*;

pub struct BarrierPlugin;

impl Plugin for BarrierPlugin {
    fn build(&self, app: &mut App) {
        // Spawn barriers on each wave start so they reset every wave.
        app.add_systems(OnEnter(GameState::Playing), spawn_barriers);
    }
}

// Classic Space Invaders bunker shape viewed from the front.
// 6 columns × 5 rows; row 0 = top, row 4 = bottom (arch cutout).
const BUNKER_SHAPE: [[bool; 6]; 5] = [
    [false, true,  true,  true,  true,  false], // row 0 – top (chamfered corners)
    [true,  true,  true,  true,  true,  true ], // row 1
    [true,  true,  true,  true,  true,  true ], // row 2
    [true,  true,  true,  true,  true,  true ], // row 3
    [true,  true,  false, false, true,  true ], // row 4 – bottom arch
];

// Two z-layers give the bunker real depth (front face toward enemies, back toward player).
const DEPTH_LAYERS: u32 = 2;

fn spawn_barriers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let block_mesh = meshes.add(Cuboid::new(
        BARRIER_BLOCK_SIZE,
        BARRIER_BLOCK_SIZE,
        BARRIER_BLOCK_SIZE,
    ));

    // Four bunkers evenly spaced across the wider arena (arena width = 18, ±9).
    let bunker_centers: [f32; 4] = [-6.75, -2.25, 2.25, 6.75];

    for &bx in &bunker_centers {
        for (row, row_shape) in BUNKER_SHAPE.iter().enumerate() {
            for (col, &present) in row_shape.iter().enumerate() {
                if !present {
                    continue;
                }

                // Horizontally center the 6-wide block grid in the bunker.
                let x = bx + (col as f32 - 2.5) * BARRIER_BLOCK_SPACING;
                // Row 0 = top, row 4 = bottom.
                let y = PLAYER_Y + (4 - row) as f32 * BARRIER_BLOCK_SPACING;

                for layer in 0..DEPTH_LAYERS {
                    // Layer 0 = front (enemy-facing, lower z); layer 1 = rear (player-facing).
                    let z = BARRIER_Z + layer as f32 * BARRIER_BLOCK_SPACING;

                    let mat = materials.add(barrier_material(BARRIER_HEALTH));

                    commands.spawn((
                        Mesh3d(block_mesh.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_xyz(x, y, z),
                        Barrier { health: BARRIER_HEALTH },
                    ));
                }
            }
        }
    }
}

/// Returns a material matching the given health level.
/// Health 2 → green (intact), health 1 → orange (damaged), 0 → should be despawned.
pub fn barrier_material(health: u8) -> StandardMaterial {
    let (color, emissive) = match health {
        2 => (
            Color::srgb(0.1, 0.9, 0.3),
            LinearRgba::new(0.2, 3.0, 0.5, 1.0),
        ),
        1 => (
            Color::srgb(1.0, 0.35, 0.05),
            LinearRgba::new(4.0, 0.8, 0.1, 1.0),
        ),
        _ => (
            Color::srgb(0.3, 0.3, 0.3),
            LinearRgba::new(0.0, 0.0, 0.0, 1.0),
        ),
    };
    StandardMaterial {
        base_color: color,
        emissive,
        metallic: 0.0,
        perceptual_roughness: 0.05,
        specular_transmission: 0.55,
        ior: 1.5,
        thickness: BARRIER_BLOCK_SIZE,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}
