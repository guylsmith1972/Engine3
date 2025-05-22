# Engine3 - Future Development Roadmap

## Introduction

With the foundational blueprint-based scene system and the core relativistic portal rendering logic now in place, Engine3 is ready for significant enhancements. This roadmap outlines key future directions to build upon this architecture, focusing on improving core mechanics, expanding rendering capabilities, and adding features that will showcase the engine's unique strengths.

The current system successfully demonstrates:
* A flexible scene definition using `HullBlueprint` and `HullInstance`.
* Relativistic portal traversal where subsequent rooms are transformed relative to the current view.
* A `SideHandler` architecture (`StandardWallHandler`, `StandardPortalHandler`).
* Correct back-face culling for interior-pointing normals, preventing unwanted recursion in simple two-way "open doorway" scenarios as specified.

This roadmap is divided into logical phases, though many items can be developed in parallel or re-prioritized as needed.

---

## Phase 1: Core Gameplay & Demo Enhancements

This phase focuses on immediate next steps to enhance the user experience and demonstrate more advanced capabilities of the portal system.

### 1. Seamless Camera Traversal Through Portals

* **Current State:** The camera can look through portals, and the engine renders the connected scenes correctly. However, the player's camera (`active_camera_instance_id` and `active_camera_local_transform`) does not yet physically transition from one hull instance to another when moving through a portal.
* **Goal:** Implement the logic for the player's camera to seamlessly move from its current `HullInstance` into a target `HullInstance` when it passes through a connecting portal.
* **Key Tasks:**
    * **Portal Collision/Triggering:** Determine when the camera has "crossed" the plane of a portal polygon that is being rendered via a `StandardPortalHandler`. This might involve simple plane intersection tests or bounding box checks for the camera.
    * **State Update:** When a portal is traversed, the `Scene` data needs to be updated:
        * `active_camera_instance_id` should change to the ID of the `target_instance_id` specified in the portal's configuration.
        * `active_camera_local_transform` (the camera's pose) needs to be updated. Its new position and orientation will be relative to the *new* hull instance's blueprint space. This transformation must be the inverse of the `portal_alignment_transform` that was used to render the view into that portal, applied to the camera's pose relative to the exit portal face. This ensures a smooth visual and positional transition.
    * **Controller Input:** Ensure the `CameraController` continues to function correctly relative to the new host hull after traversal.

### 2. Advanced Demo Scene with Recursive Effects

* **Current State:** The `demo_scene.rs` now correctly shows a non-recursive two-room setup by ensuring the connecting side of the second room is culled from further traversal, as per the specification for a simple doorway.
* **Goal:** Create a new demo scene, or extend the existing one, to explicitly showcase the engine's ability to handle recursive views (like the "hall of mirrors" effect) or simple non-Euclidean transitions.
* **Key Tasks:**
    * **Scene Design:** Define a small scene with at least two `HullInstance`s whose `StandardPortalHandler` configurations intentionally link back to each other in a loop.
    * **Verify Culling and Recursion Depth:** Ensure the `MAX_PORTAL_RECURSION_DEPTH` in `StandardPortalHandler` correctly terminates the rendering of such a scene to prevent infinite loops and crashes, while still showing several levels of recursion.
    * **Test Non-Obvious Connections:** Potentially create a portal that connects back to a different portal on the *same* hull instance, or portals that link rooms in a spatially surprising way (e.g., a short corridor that appears longer due to chained identity-alignment portals).

---

## Phase 2: Foundational Engine Improvements

This phase addresses core components that will increase the engine's robustness, flexibility, and ease of use.

### 3. Robust `Mat4` Math Library Integration

* **Current State:** The `Mat4` struct in `engine_lib/scene_types.rs` uses basic, hand-rolled implementations for matrix operations (multiplication, inverse, normal transformation). The `inverse()` method is particularly simplified and only reliable for orthonormal rotation + translation.
* **Goal:** Replace the custom `Mat4` implementation with a well-tested, feature-rich, and optimized linear algebra library (e.g., `glam` or `nalgebra-glm`).
* **Benefits:**
    * **Correctness:** Ensures accurate matrix operations, especially for `inverse()` and normal transformations (which should use the inverse transpose of the upper 3x3 for full generality with non-uniform scaling, though not currently an issue).
    * **Performance:** Leverages optimized math routines.
    * **Features:** Provides a wider array of vector and matrix operations, quaternions for rotations (which can help avoid gimbal lock and simplify complex orientation logic for the camera or animated objects), etc.
* **Key Tasks:**
    * Choose a suitable library.
    * Replace all uses of the custom `Mat4` and `Point3` (if the library provides its own vector types) throughout the codebase.
    * Update function signatures and matrix construction calls (`from_translation`, `from_rotation_x/y/z`, etc.) to use the library's API.

### 4. Generalized Portal Alignment Algorithm

* **Current State:** The `portal_alignment_transform` calculated in `StandardPortalHandler` is hardcoded with translation-only logic specific to aligning opposing faces of identical cuboid blueprints in the demo.
* **Goal:** Implement a general algorithm that can align any two arbitrary convex portal polygons.
* **Key Tasks:**
    * **Portal Frame Definition:** Decide how portal faces are defined geometrically for alignment purposes. This might involve storing a local transformation (origin and basis vectors) for each portal face within its `BlueprintSide` or deriving it from its vertices.
    * **Alignment Logic:** Given two such portal frames (one on the current hull's exit portal, one on the target hull's entry portal), calculate the `Mat4` that transforms the target hull's blueprint space into the current hull's blueprint space such that the portal frames are perfectly aligned (e.g., coincident origins, anti-parallel normals, aligned "up" vectors). This might involve steps like:
        1.  Transform to bring target portal's origin to world origin.
        2.  Rotate target portal to align its normal with the (negated) normal of the source portal.
        3.  Rotate target portal to align its "up" vector with the source portal's "up" vector.
        4.  Translate target portal (now correctly oriented at world origin) to the source portal's position.
    * This is a non-trivial geometric problem, and robust solutions (e.g., using a few corresponding points on each portal polygon) should be researched if simple frame alignment is insufficient.

### 5. Scene Authoring and Loading System

* **Current State:** Scenes are defined entirely in Rust code within `demo_scene.rs`.
* **Goal:** Enable defining and loading scenes (blueprints, instances, connections, configurations) from external data files.
* **Benefits:**
    * Allows for much faster iteration on scene design without recompiling.
    * Enables non-programmers to build or modify scenes.
    * Supports larger and more complex worlds.
* **Key Tasks:**
    * **Choose a Data Format:** Options include RON (Rusty Object Notation), JSON, YAML, or a custom binary format. RON is often a good fit for Rust projects.
    * **Define Schema:** Specify how `HullBlueprint`, `HullInstance`, `BlueprintSide`, `HandlerConfig` variants, portal connections, etc., are represented in the chosen format.
    * **Serialization/Deserialization:** Implement logic (likely using `serde`) to parse these files into the engine's runtime scene data structures.
    * Update `PolygonApp::new()` or add a new scene manager to load from a specified file instead of calling `demo_scene::create_mvp_scene()`.

---

## Phase 3: Expanding Rendering Capabilities & Features

This phase focuses on adding more visual richness and demonstrating advanced portal effects.

### 6. Advanced `SideHandler` Implementations

* **Current State:** Only `StandardWallHandler` and `StandardPortalHandler` exist.
* **Goal:** Implement more sophisticated side handlers as outlined in `docs/planning/blueprints.md`.
* **Key Handlers to Consider:**
    * **`MirrorHandler`:**
        * Calculates a reflection matrix based on the portal/mirror plane.
        * Concatenates this reflection matrix with the `accumulated_transform`.
        * Modifies the `screen_space_clip_polygon` to be the mirror's shape.
        * Re-queues the *current* hull instance for rendering with the new reflected transform and increased recursion depth. Special care is needed for the clipping plane (oblique view frustum) and winding order of reflected geometry.
    * **`CameraDisplayHandler` (Render-to-Texture):**
        * Requires setting up a secondary camera in the scene.
        * Involves a separate rendering pass of the scene (or part of it) from the secondary camera's perspective into a texture.
        * This texture is then applied to the surface of the `BlueprintSide` that uses this handler. This requires shader support for texturing.
    * **`TransparentWallHandler`:**
        * Renders a semi-transparent surface. Requires blending to be correctly configured in the WGPU pipeline.
        * May involve simple alpha blending or more complex refraction effects (which would need shader support and potentially access to a pre-rendered scene color buffer).
    * **`NonEuclideanPortalHandler`:**
        * Similar to `StandardPortalHandler` but its `portal_alignment_transform` can include non-uniform scaling or other distortions to create "bigger on the inside" effects or warped connections.

### 7. Basic Lighting Model

* **Current State:** Rendering is unlit; colors are fixed.
* **Goal:** Implement a simple lighting model to improve visual depth and realism.
* **Key Tasks:**
    * **Vertex Normals:** Ensure `Vertex` struct and blueprint geometry include 3D normals for lighting calculations (currently `local_normal` is per-face in `BlueprintSide`). Vertices themselves will need normals.
    * **Light Types:** Start with ambient light and a single directional light.
    * **Shader Updates (`shader.rs`):** Modify WGSL shaders to calculate diffuse lighting (e.g., NdotL).
    * **Uniform Buffers:** Pass light properties (direction, color, intensity) to shaders.
    * **Normal Transformation:** Ensure vertex normals are correctly transformed to world or view space in the vertex shader.

### 8. Depth Buffer Integration and Z-Sorting

* **Current State:** No depth buffer is used; rendering relies on portal traversal order and 2D clipping.
* **Goal:** Introduce depth testing for correct rendering of intersecting opaque geometry *within the same hull instance* and as a foundation for more complex effects.
* **Key Tasks:**
    * **WGPU Setup:** Configure a depth texture and depth-stencil state in the render pipeline.
    * **Shader Output:** Ensure vertex shaders correctly output clip-space Z/W for depth calculations.
    * **Clearing Depth Buffer:** Clear the depth buffer at the start of each frame, or potentially at the start of rendering each portal view if strict "painter's algorithm" through portals is desired.
    * **Transparency:** If `TransparentWallHandler` is implemented, proper Z-sorting (e.g., rendering opaque objects first, then sorted transparent objects back-to-front) or order-independent transparency techniques would be needed, often in conjunction with the depth buffer.

---

## Phase 4: Gameplay and Performance

This phase focuses on making the engine more interactive and ensuring it runs efficiently.

### 9. Basic Collision Detection and Response

* **Current State:** Camera can move freely, passing through walls.
* **Goal:** Implement basic collision detection between the camera (and potentially other dynamic objects later) and the static hull geometry.
* **Key Tasks:**
    * **Collision Shapes:** Represent camera as a simple shape (e.g., sphere, capsule, AABB). Hull sides are convex polygons.
    * **Collision Algorithm:** Implement algorithms for shape-vs-polygon intersection tests (e.g., Separating Axis Theorem for AABB vs polygon).
    * **Spatial Partitioning (Optional for now):** For larger scenes/blueprints, a spatial partitioning scheme (e.g., BSP from hull geometry, Octree) might be needed to optimize collision checks.
    * **Response:** Simple slide-along-wall response or stopping movement.
    * **Integration with Camera Controller:** Collision results should influence camera position updates.

### 10. Performance Profiling and Optimization

* **Current State:** Performance is not yet a primary focus.
* **Goal:** Establish a practice of profiling the engine and identifying/addressing bottlenecks as complexity increases.
* **Key Tasks:**
    * **Benchmarking:** Continue using `criterion` for micro-benchmarks of critical algorithms like polygon intersection and transformations.
    * **Frame Profiling Tools:** Utilize tools like `Tracy`, WGPU's built-in debugging/profiling features, or platform-specific profilers to analyze frame times and identify hotspots in CPU or GPU usage.
    * **Optimization Targets:**
        * Matrix math (especially if still custom).
        * Polygon clipping algorithms.
        * Number of draw calls (though portal culling helps significantly here).
        * Shader complexity.
        * Data copying between CPU and GPU.

---

This roadmap provides a comprehensive overview of potential next steps. The order and priority can certainly be adjusted based on your specific goals for Engine3. Good luck!