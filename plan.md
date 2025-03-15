# 14-Hour Voxel Game

## ✅ Install Bevy + Plugins

```bash
cargo add bevy bevy_egui noise rand
```

✅ Set up a Bevy App

    Basic empty 3D scene.
    FPS Camera with movement (Transform).

## 2️⃣ Procedural Terrain (4 Hours)

✅ Chunk System (1.5h)

    Store chunks as HashMap<(x, y, z), BlockType>.
    Keep a 3x3 grid of active chunks.

✅ Perlin Noise Generation (2h)

    Use noise crate for heightmaps.
    Generate grass, stone, and dirt layers.

✅ Basic Meshing (0.5h)

    Implement greedy meshing to merge adjacent blocks.

## 3️⃣ Lighting (3 Hours)

✅ Ambient Occlusion (1.5h)

    Fake lighting by darkening inner faces.

✅ Directional Light (1.5h)

    Add a sunlight source (adjust based on time of day).

## 4️⃣ Voxel Interaction & Gameplay (3 Hours)

✅ Raycasting (1.5h)

    Detect block under crosshair and remove/place blocks.

✅ Player Collision & Gravity (1.5h)

    Use AABB collision detection with voxels.
    Apply simple gravity (move downward unless on solid ground).

## 5️⃣ AI-Generated Assets (2.5 Hours)

✅ Textures (1h)

    Use Leonardo.Ai / Stable Diffusion to generate block textures.

✅ Sounds (1h)

    AI-generated block breaking/placing sounds from Freesound/Riffusion.

✅ Skybox (0.5h)

    Generate a simple sky texture for a nice atmosphere.

Final output:
- [ ] Procedural terrain
- [ ] Lighting (ambient + sun)
- [ ] Fast rendering (meshing + culling)
- [ ] Player movement & physics
