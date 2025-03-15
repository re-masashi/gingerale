use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_asset::{AssetApp, RenderAssetUsages};
use noise::{NoiseFn, Perlin};
use rand::Rng;

use std::collections::{HashMap, HashSet};

const CHUNK_SIZE: usize = 40;
const WORLD_HEIGHT: usize = 128; // Increase height for better terrain
const NOISE_SCALE: f64 = 0.0314; // Controls terrain smoothness
const RENDER_DISTANCE: i32 = 5; // Load chunks in a n x n area around the player

#[derive(Resource)]
struct ChunkManager {
    loaded_chunks: HashSet<IVec2>,  // Stores loaded chunk positions
    chunks: HashMap<IVec2, Entity>, // Maps chunk positions to Bevy entities
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            loaded_chunks: HashSet::new(),
            chunks: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BlockType {
    Air,
    Solid,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Biome {
    Grassland,
    Desert,
    Snow,
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct Chunk {
    blocks: [[[BlockType; CHUNK_SIZE]; WORLD_HEIGHT]; CHUNK_SIZE], // 3D array of blocks
    position: IVec3, // Chunk position in chunk coordinates
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ChunkManager::default()) // ✅ Track chunks
        .add_systems(Startup, setup)
        .add_systems(Startup, generate_initial_chunks)
        .add_systems(
            Update,
            (
                move_camera,
                update_chunks,
                break_block,
                place_block,
                update_sun,
            ),
        ) // ✅ Now includes block placing
        .run();
}

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    mut _windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    // // circular base
    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(4.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    // ));
    // // cube
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    // ));
    // light
    // commands.spawn((
    //     PointLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    // ));

    // // Lock cursor (Bevy 0.12+)
    // if let Ok(mut window) = windows.get_single_mut() {
    //     window.cursor_options.visible = false;
    //     window.cursor_options.grab_mode = CursorGrabMode::Locked;
    // }

    // // Spawn the camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));

    // Spawn the camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    // ✅ Spawn the Sun (Directional Light)
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 50_000.0, // Brightness of the sun
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 100.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Sun, // ✅ Tag it as the Sun
    ));
}

fn move_camera(
    mut query: Query<&mut Transform, (With<Camera3d>,)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let mut transform = query.single_mut();

    let speed = 5.0;
    let sensitivity = 0.1;

    // WASD Movement
    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        direction.y -= 1.0;
    }

    let forward = transform.forward();
    let right = transform.right();
    let movement = (forward * direction.z + right * direction.x + Vec3::Y * direction.y)
        .normalize_or_zero()
        * speed
        * time.delta_secs();

    transform.translation += movement;

    // Mouse Movement
    for event in motion_events.read() {
        let delta_x = event.delta.x;
        let delta_y = event.delta.y;

        let rotation_x = Quat::from_rotation_y(-delta_x * sensitivity * time.delta_secs());
        let rotation_y = Quat::from_rotation_x(-delta_y * sensitivity * time.delta_secs());

        transform.rotation = rotation_x * transform.rotation * rotation_y;
    }
}

fn generate_initial_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk_pos = IVec2::new(0, 0); // Start with a single chunk at (0,0)
    generate_chunk(&mut commands, &mut meshes, &mut materials, chunk_pos);
}

// Modify `generate_chunk` to use biomes

fn generate_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_pos: IVec2,
) -> Entity {
    let mut rng = rand::rng();
    let terrain_noise = Perlin::new(rng.random());
    let cave_noise = Perlin::new(rng.random());

    let mut chunk = Chunk {
        blocks: [[[BlockType::Air; CHUNK_SIZE]; WORLD_HEIGHT]; CHUNK_SIZE],
        position: IVec3::new(chunk_pos.x, 0, chunk_pos.y),
    };

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let world_x = chunk.position.x as f64 * CHUNK_SIZE as f64 + x as f64;
            let world_z = chunk.position.z as f64 * CHUNK_SIZE as f64 + z as f64;

            // **Step 1: Determine Biome**
            let biome_value = terrain_noise.get([world_x * 0.005, world_z * 0.005]);
            let biome = if biome_value < -0.2 {
                Biome::Desert
            } else if biome_value > 0.4 {
                Biome::Snow
            } else {
                Biome::Grassland
            };

            // **Step 2: Generate Terrain Heightmap**
            let height = (terrain_noise.get([world_x * NOISE_SCALE, world_z * NOISE_SCALE]) * 15.0
                + 20.0) as usize;
            let clamped_height = height.min(WORLD_HEIGHT - 1);

            for y in 0..=clamped_height {
                let world_y = y as f64;

                // **Step 3: Generate Caves**
                let cave_value = cave_noise.get([world_x * 0.1, world_y * 0.1, world_z * 0.1]);
                let is_cave = cave_value > -0.3;

                if is_cave {
                    chunk.blocks[x][y][z] = match biome {
                        Biome::Grassland => {
                            if y == clamped_height {
                                BlockType::Solid // Grass block
                            } else {
                                BlockType::Solid // Dirt
                            }
                        }
                        Biome::Desert => BlockType::Solid, // Sand
                        Biome::Snow => BlockType::Solid,   // Snow block
                    };
                }
            }

            // **Step 4: Generate Trees in Grassland**
            if biome == Biome::Grassland
                && rng.gen_range(0..100) < 5
                && clamped_height < WORLD_HEIGHT - 6
            {
                for i in 0..5 {
                    chunk.blocks[x][clamped_height + i][z] = BlockType::Solid; // Trunk
                }
                for dx in -2..=2 {
                    for dz in -2..=2 {
                        for dy in 3..=5 {
                            if rng.gen_range(0..100) < 70 {
                                let leaf_x =
                                    (x as i32 + dx).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                                let leaf_y = (clamped_height + dy).clamp(0, WORLD_HEIGHT - 1);
                                let leaf_z =
                                    (z as i32 + dz).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                                chunk.blocks[leaf_x][leaf_y][leaf_z] = BlockType::Solid;
                                // Leaves
                            }
                        }
                    }
                }
            }
        }
    }

    let entity = spawn_chunk(commands, meshes, materials, &chunk);
    entity
}
// fn spawn_chunk(
//     commands: &mut Commands,
//     meshes: &mut ResMut<Assets<Mesh>>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
//     chunk: &Chunk,
// ) {
//     let mut mesh_data = Vec::new();
//     let mut indices = Vec::new();
//     let mut index_offset = 0;

//     let chunk_offset = chunk.position * CHUNK_SIZE as i32;

//     for x in 0..CHUNK_SIZE {
//         for y in 0..CHUNK_SIZE {
//             for z in 0..CHUNK_SIZE {
//                 if chunk.blocks[x][y][z] == BlockType::Solid {
//                     for (dx, dy, dz, normal) in [
//                         (1, 0, 0, Vec3::X), (-1, 0, 0, -Vec3::X),
//                         (0, 1, 0, Vec3::Y), (0, -1, 0, -Vec3::Y),
//                         (0, 0, 1, Vec3::Z), (0, 0, -1, -Vec3::Z),
//                     ] {
//                         let neighbor_x = (x as i32 + dx) as usize;
//                         let neighbor_y = (y as i32 + dy) as usize;
//                         let neighbor_z = (z as i32 + dz) as usize;

//                         let is_out_of_bounds = neighbor_x >= CHUNK_SIZE || neighbor_y >= CHUNK_SIZE || neighbor_z >= CHUNK_SIZE;
//                         let is_air = !is_out_of_bounds && chunk.blocks[neighbor_x][neighbor_y][neighbor_z] == BlockType::Air;

//                         if is_out_of_bounds || is_air {
//                             let world_pos = Vec3::new(
//                                 (x as i32 + chunk_offset.x) as f32,
//                                 (y as i32 + chunk_offset.y) as f32,
//                                 (z as i32 + chunk_offset.z) as f32,
//                             );

//                             let vertices = [
//                                 world_pos + Vec3::new(0.0, 0.0, 0.0),
//                                 world_pos + Vec3::new(1.0, 0.0, 0.0),
//                                 world_pos + Vec3::new(1.0, 1.0, 0.0),
//                                 world_pos + Vec3::new(0.0, 1.0, 0.0),
//                             ];

//                             mesh_data.extend_from_slice(&vertices);
//                             indices.extend_from_slice(&[
//                                 index_offset, index_offset + 1, index_offset + 2,
//                                 index_offset, index_offset + 2, index_offset + 3
//                             ]);

//                             index_offset += 4;
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     let mut mesh = Mesh::new(
//         bevy::render::render_resource::PrimitiveTopology::TriangleList,
//         RenderAssetUsages::default(), // ❌ This no longer works
//     );
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data);
//     mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

//     let mesh_handle = meshes.add(mesh);
//     let material_handle = materials.add(StandardMaterial {
//         base_color: Color::rgb(0.6, 0.7, 0.8),
//         ..default()
//     });

//     commands.spawn(PbrBundle {
//         mesh: Mesh3d(mesh_handle),
//         material: MeshMaterial3d(material_handle),
//         transform: Transform::default(),
//         ..default()
//     });
// }

// Define all 6 possible face directions
const FACE_DIRECTIONS: [(IVec3, [Vec3; 4]); 6] = [
    (
        IVec3::new(1, 0, 0),
        [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ],
    ),
    (
        IVec3::new(-1, 0, 0),
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ],
    ),
    (
        IVec3::new(0, 1, 0),
        [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
    ),
    (
        IVec3::new(0, -1, 0),
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
        ],
    ),
    (
        IVec3::new(0, 0, 1),
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
        ],
    ),
    (
        IVec3::new(0, 0, -1),
        [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
    ),
];

fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk: &Chunk,
) -> Entity {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index_count = 0;

    let chunk_offset = chunk.position * CHUNK_SIZE as i32;

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if chunk.blocks[x][y][z] == BlockType::Solid {
                    let block_pos = IVec3::new(x as i32, y as i32, z as i32);

                    for (normal, face) in FACE_DIRECTIONS.iter() {
                        let neighbor_pos = block_pos + *normal;
                        let is_out_of_bounds = neighbor_pos.x < 0
                            || neighbor_pos.y < 0
                            || neighbor_pos.z < 0
                            || neighbor_pos.x >= CHUNK_SIZE as i32
                            || neighbor_pos.y >= CHUNK_SIZE as i32
                            || neighbor_pos.z >= CHUNK_SIZE as i32;
                        let is_air = is_out_of_bounds
                            || chunk.blocks[neighbor_pos.x as usize][neighbor_pos.y as usize]
                                [neighbor_pos.z as usize]
                                == BlockType::Air;

                        if is_air {
                            let world_pos = Vec3::new(
                                (block_pos.x + chunk_offset.x) as f32,
                                (block_pos.y + chunk_offset.y) as f32,
                                (block_pos.z + chunk_offset.z) as f32,
                            );

                            let face_vertices: Vec<Vec3> =
                                face.iter().map(|v| *v + world_pos).collect();
                            vertices.extend_from_slice(&face_vertices);

                            indices.extend_from_slice(&[
                                index_count,
                                index_count + 1,
                                index_count + 2,
                                index_count,
                                index_count + 2,
                                index_count + 3,
                            ]);

                            index_count += 4;
                        }
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.6, 0.7, 0.8),
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(material_handle),
            transform: Transform::default(),
            ..default()
        })
        .id()
}

fn update_chunks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Camera3d>>,
) {
    let player_transform = player_query.single();
    let player_chunk_x = (player_transform.translation.x / CHUNK_SIZE as f32).floor() as i32;
    let player_chunk_z = (player_transform.translation.z / CHUNK_SIZE as f32).floor() as i32;

    let mut new_chunks = HashSet::new();

    // Load chunks in a square around the player
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = IVec2::new(player_chunk_x + dx, player_chunk_z + dz);
            new_chunks.insert(chunk_pos);
        }
    }

    // Unload chunks that are too far
    let chunks_to_unload: Vec<_> = chunk_manager
        .loaded_chunks
        .difference(&new_chunks)
        .cloned()
        .collect();
    for chunk_pos in chunks_to_unload {
        if let Some(entity) = chunk_manager.chunks.remove(&chunk_pos) {
            commands.entity(entity).despawn_recursive();
        }
        chunk_manager.loaded_chunks.remove(&chunk_pos);
    }

    // Load new chunks
    let chunks_to_load: Vec<_> = new_chunks
        .difference(&chunk_manager.loaded_chunks)
        .cloned()
        .collect();
    for chunk_pos in chunks_to_load {
        let entity = generate_chunk(&mut commands, &mut meshes, &mut materials, chunk_pos);
        chunk_manager.chunks.insert(chunk_pos, entity);
        chunk_manager.loaded_chunks.insert(chunk_pos);
    }
}

fn raycast_block(
    camera_transform: &Transform,
    chunk_manager: &ChunkManager,
    chunk_query: &Query<&Chunk>,
) -> Option<(IVec3, IVec3, IVec2)> {
    let max_distance = 5.0;
    let step_size = 0.1;

    let mut ray_pos = camera_transform.translation;
    let ray_dir = camera_transform.forward();

    let mut last_air_pos = None; // Store the last empty block position

    for _ in 0..(max_distance / step_size) as i32 {
        let block_pos = IVec3::new(
            ray_pos.x.floor() as i32,
            ray_pos.y.floor() as i32,
            ray_pos.z.floor() as i32,
        );

        let chunk_pos = IVec2::new(
            block_pos.x / CHUNK_SIZE as i32,
            block_pos.z / CHUNK_SIZE as i32,
        );

        if let Some(entity) = chunk_manager.chunks.get(&chunk_pos) {
            if let Ok(chunk) = chunk_query.get(*entity) {
                let local_x = (block_pos.x.rem_euclid(CHUNK_SIZE as i32)) as usize;
                let local_y = (block_pos.y.rem_euclid(WORLD_HEIGHT as i32)) as usize;
                let local_z = (block_pos.z.rem_euclid(CHUNK_SIZE as i32)) as usize;

                if chunk.blocks[local_x][local_y][local_z] == BlockType::Solid {
                    return last_air_pos.map(|air_pos| (block_pos, air_pos, chunk_pos));
                // ✅ Return the first air position
                } else {
                    last_air_pos = Some(block_pos); // ✅ Store the last air block
                }
            }
        }

        ray_pos += ray_dir * step_size;
    }

    None
}

fn break_block(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut chunk_queries: ParamSet<(Query<&Chunk>, Query<&mut Chunk>)>, // ✅ Fix: Use ParamSet to separate read/write
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let camera_transform = camera_query.single();

        // Use first query set (immutable) for raycasting
        if let Some((_, block_pos, chunk_pos)) =
            raycast_block(camera_transform, &chunk_manager, &chunk_queries.p0())
        {
            if let Some(entity) = chunk_manager.chunks.get(&chunk_pos) {
                // Use second query set (mutable) to modify the chunk
                if let Ok(mut chunk) = chunk_queries.p1().get_mut(*entity) {
                    let local_x = (block_pos.x.rem_euclid(CHUNK_SIZE as i32)) as usize;
                    let local_y = (block_pos.y.rem_euclid(WORLD_HEIGHT as i32)) as usize;
                    let local_z = (block_pos.z.rem_euclid(CHUNK_SIZE as i32)) as usize;

                    chunk.blocks[local_x][local_y][local_z] = BlockType::Air; // ✅ Remove block

                    // Regenerate the chunk mesh
                    commands.entity(*entity).despawn_recursive();
                    let new_entity =
                        spawn_chunk(&mut commands, &mut meshes, &mut materials, &chunk);
                    chunk_manager.chunks.insert(chunk_pos, new_entity);
                }
            }
        }
    }
}

fn place_block(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut chunk_queries: ParamSet<(Query<&Chunk>, Query<&mut Chunk>)>, // ✅ Fix conflicting queries
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        let camera_transform = camera_query.single();

        if let Some((_solid_block_pos, air_pos, chunk_pos)) =
            raycast_block(camera_transform, &chunk_manager, &chunk_queries.p0())
        {
            if let Some(entity) = chunk_manager.chunks.get(&chunk_pos) {
                if let Ok(mut chunk) = chunk_queries.p1().get_mut(*entity) {
                    let local_x = (air_pos.x.rem_euclid(CHUNK_SIZE as i32)) as usize;
                    let local_y = (air_pos.y.rem_euclid(WORLD_HEIGHT as i32)) as usize;
                    let local_z = (air_pos.z.rem_euclid(CHUNK_SIZE as i32)) as usize;

                    chunk.blocks[local_x][local_y][local_z] = BlockType::Solid; // ✅ Place block

                    // Regenerate the chunk mesh
                    commands.entity(*entity).despawn_recursive();
                    let new_entity =
                        spawn_chunk(&mut commands, &mut meshes, &mut materials, &chunk);
                    chunk_manager.chunks.insert(chunk_pos, new_entity);
                }
            }
        }
    }
}

fn update_sun(time: Res<Time>, mut sun_query: Query<&mut Transform, With<Sun>>) {
    let mut sun_transform = sun_query.single_mut();

    let speed = 0.1; // How fast the sun moves (day length coefficent)
    let rotation = Quat::from_rotation_x(speed * time.delta_secs());

    sun_transform.rotate(rotation);
}
