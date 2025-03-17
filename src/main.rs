use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_asset::RenderAssetUsages;
use bevy_fps_counter::{FpsCounter, FpsCounterPlugin};
use bevy_skybox::{SkyboxCamera, SkyboxPlugin};
use noise::{NoiseFn, Perlin};
use rand::Rng;

use std::collections::{HashMap, HashSet};

const CHUNK_SIZE: usize = 40;
const WORLD_HEIGHT: usize = 256; // Increase height for better terrain
const NOISE_SCALE: f64 = 0.01; // Controls terrain smoothness
const RENDER_DISTANCE: i32 = 5; // Load chunks in a n x n area around the player

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FollowPlayer {
    target: Entity, // The player entity to follow
}

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
    Solid(u8),
    // 0 -> dirt
    // 1 -> grass
    // 2 -> sand
    // 3 -> snow
}

#[derive(Resource)]
struct BlockTextureAtlas {
    // _layout: Handle<TextureAtlasLayout>,
    image: Handle<Image>,
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

fn follow_player_system(
    players: Query<&Transform, With<Player>>,
    mut cameras: Query<(&mut Transform, &FollowPlayer)>,
) {
    for (mut cam_transform, follow) in cameras.iter_mut() {
        if let Ok(player_transform) = players.get(follow.target) {
            // Smoothly interpolate towards the player's position
            let target_position = player_transform.translation + Vec3::new(0.0, 5.0, 10.0);
            cam_transform.translation = cam_transform.translation.lerp(target_position, 0.1);

            // Make camera look at the player
            cam_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsCounterPlugin)
        .add_plugins(SkyboxPlugin::from_image_file("sky1.png"))
        .insert_resource(ChunkManager::default()) // ✅ Track chunks
        .add_systems(Startup, setup)
        .add_systems(Startup, generate_initial_chunks.after(setup))
        .add_systems(
            Update,
            (
                move_camera,
                update_chunks,
                // break_block,
                // place_block,
                update_sun,
                mouse_handler,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ambient_light: ResMut<AmbientLight>, // ✅ Add AmbientLight
) {
    let texture_handle = asset_server.load("textures/atlas.png");
    let _layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 4, 6, None, None);
    // let layout_handle = layouts.add(layout);

    // ✅ Increase ambient light intensity
    *ambient_light = AmbientLight {
        color: Color::srgb(1.0, 1.0, 1.0), // White light
        brightness: 1700.0,                // Adjust as needed
    };

    commands.insert_resource(BlockTextureAtlas {
        // layout: layout_handle,
        image: texture_handle,
    });

    let player = commands
        .spawn(PbrBundle {
            mesh: bevy::prelude::Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
            material: bevy::prelude::MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/grass.png")),
                ..default()
            })),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        })
        .insert(Player)
        .id();

    commands
        .spawn((
            Camera3d::default(),
            Projection::from(PerspectiveProjection {
                // 120 degree field-of-view.
                fov: 70.0_f32.to_radians(),
                ..default()
            }),
            Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            SkyboxCamera,
        ))
        .insert(FollowPlayer { target: player });

    // Spawn the Sun
    commands.spawn((
        DirectionalLight {
            illuminance: 150_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun, // ✅ Tag it as the Sun
    ));

    commands.spawn(PointLight {
        intensity: 50000.0, // ✅ Soft bounce light
        radius: 20000.0,
        color: Color::srgb(1.0, 0.95, 0.8), // Warm light
        ..default()
    });
}

fn mouse_handler(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut diags_state: ResMut<FpsCounter>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        if diags_state.is_enabled() {
            diags_state.disable();
        } else {
            diags_state.enable();
        }
    }
}

fn move_camera(
    mut query: Query<&mut Transform, (With<Camera3d>,)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let mut transform = query.single_mut();

    // let speed = 5.0;
    let sensitivity = 0.1;

    // // WASD Movement
    // let mut direction = Vec3::ZERO;
    // if keyboard_input.pressed(KeyCode::KeyW) {
    //     direction.z += 1.0;
    // }
    // if keyboard_input.pressed(KeyCode::KeyS) {
    //     direction.z -= 1.0;
    // }
    // if keyboard_input.pressed(KeyCode::KeyA) {
    //     direction.x -= 1.0;
    // }
    // if keyboard_input.pressed(KeyCode::KeyD) {
    //     direction.x += 1.0;
    // }
    // if keyboard_input.pressed(KeyCode::Space) {
    //     direction.y += 1.0;
    // }
    // if keyboard_input.pressed(KeyCode::ShiftLeft) {
    //     direction.y -= 1.0;
    // }

    // let forward = transform.forward();
    // let right = transform.right();
    // let movement = (forward * direction.z + right * direction.x + Vec3::Y * direction.y)
    //     .normalize_or_zero()
    //     * speed
    //     * time.delta_secs();

    // transform.translation += movement;

    // Mouse Movement
    for event in motion_events.read() {
        let delta_x = event.delta.x;
        let delta_y = event.delta.y;

        let rotation_x = Quat::from_rotation_y(-delta_x * sensitivity * time.delta_secs());
        let rotation_y = Quat::from_rotation_x(-delta_y * 0.0 * time.delta_secs());

        transform.rotation = rotation_x * transform.rotation * rotation_y;
    }
    let mut player_transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += Vec3::new(0.0, 0.0, -1.0);
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += Vec3::new(0.0, 0.0, 1.0);
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += Vec3::new(-1.0, 0.0, 0.0);
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::new(1.0, 0.0, 0.0);
    }
    if keyboard.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        direction.y -= 1.0;
    }

    let speed = 5.0;
    player_transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();
}

fn generate_initial_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    textures: Res<BlockTextureAtlas>, // ✅ Add textures here
) {
    let chunk_pos = IVec2::new(0, 0); // Start with a single chunk at (0,0)
    generate_chunk(
        &mut commands,
        &mut meshes,
        &mut materials,
        &textures,
        chunk_pos,
    );
}

fn generate_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    textures: &Res<BlockTextureAtlas>, // ✅ Add textures here
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
                                BlockType::Solid(1) // Grass block
                            } else {
                                BlockType::Solid(0) // Dirt
                            }
                        }
                        Biome::Desert => BlockType::Solid(2), // Sand
                        Biome::Snow => BlockType::Solid(3),   // Snow block
                    };
                }
            }

            // **Step 4: Generate Trees in Grassland**
            if biome == Biome::Grassland
                && rng.random_range(0..1000) < 5
                && clamped_height < WORLD_HEIGHT - 6
            {
                for i in 0..5 {
                    chunk.blocks[x][clamped_height + i][z] = BlockType::Solid(0);
                    // Trunk
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
                                chunk.blocks[leaf_x][leaf_y][leaf_z] = BlockType::Solid(1);
                                // Leaves
                            }
                        }
                    }
                }
            }
        }
    }

    let entity = spawn_chunk(commands, meshes, materials, textures, &chunk);
    entity
}

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

// fn spawn_chunk(
//     commands: &mut Commands,
//     meshes: &mut ResMut<Assets<Mesh>>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
//     chunk: &Chunk,
// ) -> Entity {
//     let mut vertices = Vec::new();
//     let mut indices = Vec::new();
//     let mut index_count = 0;

//     let chunk_offset = chunk.position * CHUNK_SIZE as i32;

//     for x in 0..CHUNK_SIZE {
//         for y in 0..CHUNK_SIZE {
//             for z in 0..CHUNK_SIZE {
//                 if let BlockType::Solid(solid_type) = chunk.blocks[x][y][z] {
//                     let block_pos = IVec3::new(x as i32, y as i32, z as i32);

//                     for (normal, face) in FACE_DIRECTIONS.iter() {
//                         let neighbor_pos = block_pos + *normal;
//                         let is_out_of_bounds = neighbor_pos.x < 0
//                             || neighbor_pos.y < 0
//                             || neighbor_pos.z < 0
//                             || neighbor_pos.x >= CHUNK_SIZE as i32
//                             || neighbor_pos.y >= CHUNK_SIZE as i32
//                             || neighbor_pos.z >= CHUNK_SIZE as i32;
//                         let is_air = is_out_of_bounds
//                             || chunk.blocks[neighbor_pos.x as usize][neighbor_pos.y as usize]
//                                 [neighbor_pos.z as usize]
//                                 == BlockType::Air;

//                         if is_air {
//                             let world_pos = Vec3::new(
//                                 (block_pos.x + chunk_offset.x) as f32,
//                                 (block_pos.y + chunk_offset.y) as f32,
//                                 (block_pos.z + chunk_offset.z) as f32,
//                             );

//                             let face_vertices: Vec<Vec3> =
//                                 face.iter().map(|v| *v + world_pos).collect();
//                             vertices.extend_from_slice(&face_vertices);

//                             indices.extend_from_slice(&[
//                                 index_count,
//                                 index_count + 1,
//                                 index_count + 2,
//                                 index_count,
//                                 index_count + 2,
//                                 index_count + 3,
//                             ]);

//                             index_count += 4;
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     let mut mesh = Mesh::new(
//         bevy::render::render_resource::PrimitiveTopology::TriangleList,
//         RenderAssetUsages::default(),
//     );
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
//     mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

//     let mesh_handle = meshes.add(mesh);

//     // Assign block color based on type
//     let material_handle = materials.add(StandardMaterial {
//         base_color: match chunk.blocks[0][0][0] {
//             // BlockType::Solid(0) => Color::rgb(0.545, 0.271, 0.075), // Brown (Dirt)
//             BlockType::Solid(0) | BlockType::Solid(1) => Color::srgb(0.133, 0.545, 0.133), // Green (Grass)
//             BlockType::Solid(2) => Color::srgb(1.0, 0.843, 0.0), // Yellow (Sand)
//             BlockType::Solid(3) => Color::srgb(1.0, 1.0, 1.0),   // White (Snow)
//             _ => Color::srgb(0.6, 0.7, 0.8),                     // Default color
//         },
//         ..default()
//     });

//     commands
//         .spawn(PbrBundle {
//             mesh: Mesh3d(mesh_handle),
//             material: MeshMaterial3d(material_handle),
//             transform: Transform::default(),
//             ..default()
//         })
//         .id()
// }

fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    texture_atlas: &Res<BlockTextureAtlas>,
    chunk: &Chunk,
) -> Entity {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new(); // Store UV coordinates
    let mut index_count = 0;

    let chunk_offset = chunk.position * CHUNK_SIZE as i32;
    let cell_size = 1.0 / 4.0; // 4 columns for blocks
    let face_size = 1.0 / 6.0; // 6 rows for faces

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if let BlockType::Solid(block_id) = chunk.blocks[x][y][z] {
                    let block_pos = IVec3::new(x as i32, y as i32, z as i32);

                    for (face_index, (normal, face)) in FACE_DIRECTIONS.iter().enumerate() {
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

                            let uv_x = block_id as f32 * cell_size;
                            let uv_y = face_index as f32 * face_size;

                            uvs.extend_from_slice(&[
                                Vec2::new(uv_x, uv_y),
                                Vec2::new(uv_x + cell_size, uv_y),
                                Vec2::new(uv_x + cell_size, uv_y + face_size),
                                Vec2::new(uv_x, uv_y + face_size),
                            ]);

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
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_atlas.image.clone()),
        base_color: Color::srgb(1.0, 1.0, 1.0),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.5, // Adjust as needed
        unlit: false,     // Ensure it's not set to "unlit"
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
    textures: Res<BlockTextureAtlas>, // ✅ Add textures here
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
    // let mut textures = bevy::prelude::Res::<'_, BlockTextureAtlas>::clone(&textures);
    for chunk_pos in chunks_to_load {
        let entity = generate_chunk(
            &mut commands,
            &mut meshes,
            &mut materials,
            &textures,
            chunk_pos,
        );
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

                if let BlockType::Solid(_) = chunk.blocks[local_x][local_y][local_z] {
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

// fn break_block(
//     mut commands: Commands,
//     mut chunk_manager: ResMut<ChunkManager>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mouse_input: Res<ButtonInput<MouseButton>>,
//     camera_query: Query<&Transform, With<Camera3d>>,
//     // textures: Res<BlockTextureAtlas>, // ✅ Add textures here
//     mut chunk_queries: ParamSet<(Query<&Chunk>, Query<&mut Chunk>)>, // ✅ Fix: Use ParamSet to separate read/write
// ) {
//     if mouse_input.just_pressed(MouseButton::Left) {
//         let camera_transform = camera_query.single();

//         // Use first query set (immutable) for raycasting
//         if let Some((_, block_pos, chunk_pos)) =
//             raycast_block(camera_transform, &chunk_manager, &chunk_queries.p0())
//         {
//             if let Some(entity) = chunk_manager.chunks.get(&chunk_pos) {
//                 // Use second query set (mutable) to modify the chunk
//                 if let Ok(mut chunk) = chunk_queries.p1().get_mut(*entity) {
//                     let local_x = (block_pos.x.rem_euclid(CHUNK_SIZE as i32)) as usize;
//                     let local_y = (block_pos.y.rem_euclid(WORLD_HEIGHT as i32)) as usize;
//                     let local_z = (block_pos.z.rem_euclid(CHUNK_SIZE as i32)) as usize;

//                     chunk.blocks[local_x][local_y][local_z] = BlockType::Air; // ✅ Remove block

//                     // Regenerate the chunk mesh
//                     commands.entity(*entity).despawn_recursive();
//                     let new_entity = spawn_chunk(
//                         &mut commands,
//                         &mut meshes,
//                         &mut materials,
//                         // &textures,
//                         &chunk,
//                     );
//                     chunk_manager.chunks.insert(chunk_pos, new_entity);
//                 }
//             }
//         }
//     }
// }

// fn place_block(
//     mut commands: Commands,
//     mut chunk_manager: ResMut<ChunkManager>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mouse_input: Res<ButtonInput<MouseButton>>,
//     camera_query: Query<&Transform, With<Camera3d>>,
//     // textures: Res<BlockTextureAtlas>, // ✅ Add textures here
//     mut chunk_queries: ParamSet<(Query<&Chunk>, Query<&mut Chunk>)>, // ✅ Fix conflicting queries
// ) {
//     if mouse_input.just_pressed(MouseButton::Right) {
//         let camera_transform = camera_query.single();

//         if let Some((_solid_block_pos, air_pos, chunk_pos)) =
//             raycast_block(camera_transform, &chunk_manager, &chunk_queries.p0())
//         {
//             if let Some(entity) = chunk_manager.chunks.get(&chunk_pos) {
//                 if let Ok(mut chunk) = chunk_queries.p1().get_mut(*entity) {
//                     let local_x = (air_pos.x.rem_euclid(CHUNK_SIZE as i32)) as usize;
//                     let local_y = (air_pos.y.rem_euclid(WORLD_HEIGHT as i32)) as usize;
//                     let local_z = (air_pos.z.rem_euclid(CHUNK_SIZE as i32)) as usize;

//                     chunk.blocks[local_x][local_y][local_z] = BlockType::Solid(2); // ✅ Place block

//                     // Regenerate the chunk mesh
//                     commands.entity(*entity).despawn_recursive();
//                     let new_entity = spawn_chunk(
//                         &mut commands,
//                         &mut meshes,
//                         &mut materials,
//                         // &textures,
//                         &chunk,
//                     );
//                     chunk_manager.chunks.insert(chunk_pos, new_entity);
//                 }
//             }
//         }
//     }
// }

fn update_sun(time: Res<Time>, mut sun_query: Query<&mut Transform, With<Sun>>) {
    let mut sun_transform = sun_query.single_mut();

    let speed = 0.1; // How fast the sun moves (day length coefficent)
    let rotation = Quat::from_rotation_x(speed * time.delta_secs());

    sun_transform.rotate(rotation);
}
