# Design Document: Advanced Hull & Portal System

## 1. Introduction & Goals

This document outlines the design for a new system for representing and rendering hulls (convex spatial regions) within the 3D engine. The goal is to move beyond a static, world-space definition of hulls to a more flexible, memory-efficient, and powerful system that supports advanced rendering effects and complex spatial relationships.

### 1.1. Current Limitations (Implied)

* **Geometric Repetition**: Defining unique geometry for every hull instance is inefficient if many hulls share the same shape.
* **Limited Spatial Effects**: A fixed world-space representation makes it difficult to implement effects like true mirrors, non-Euclidean geometries (e.g., "bigger on the inside" rooms), or seamlessly integrated in-game camera views.

### 1.2. Core Objectives of the New System

1.  **Memory Efficiency**: Allow multiple hull instances to share common geometry via a "blueprint" system.
2.  **Design Flexibility**: Enable complex scene construction by instancing and connecting hull blueprints in varied ways.
3.  **Advanced Spatial Effects**: Support the implementation of:
    * Mirrors.
    * In-game camera views rendered onto surfaces.
    * Non-Euclidean geometries and "wormhole" type portal connections.
4.  **Extensible Side Behaviors**: Move beyond simple "wall" or "portal" sides to a system where side behaviors are defined by pluggable "Side Handlers" with instance-specific configurations.
5.  **Relativistic Coordinates**: Adopt a rendering approach where geometry is transformed relative to the camera's current hull and portal traversal path, rather than relying solely on a global, static world space.

## 2. Core Architectural Concepts

### 2.1. Relativistic Hull-Space Coordinates

Instead of all geometry being fixed in a single world coordinate system, the rendering process will be based on coordinates relative to the hull the camera currently occupies.

* **2.1.1. Camera Perspective**: The camera's view matrix is primarily defined by its position and orientation *within the local space of its current hull blueprint*. This hull's space is the initial frame of reference.
* **2.1.2. Portal Traversal and Relative Transformations**: When rendering through a portal from a current hull (Hull A) to a neighboring hull (Hull B):
    * The geometry of Hull B (defined in its own blueprint's local space) is dynamically transformed to align its entry portal with Hull A's exit portal.
    * This alignment transformation is concatenated with the transformation already applied to view Hull A.
    * This creates a chain of transformations, effectively bringing the content of subsequent hulls into the camera's initial view space without requiring a common global world space for all static geometry.

### 2.2. Hull Blueprints

* **2.2.1. Purpose**: To define the intrinsic, reusable geometry and default side properties of a type of hull (e.g., "corridor section," "small room," "large atrium").
* **2.2.2. Data Structure (Conceptual)**:
    * `blueprint_id`: Unique identifier.
    * `local_vertices`: A `Vec<Point3>` defining all vertices of the hull in its own local coordinate system (e.g., centered around origin).
    * `blueprint_sides`: A `Vec<BlueprintSide>` defining each face, its geometry (via indices into `local_vertices`), local normal, default handler type, and default handler configuration.

### 2.3. Hull Instances

* **2.3.1. Purpose**: To represent a specific occurrence of a `HullBlueprint` within the scene, defining its connections and any deviations from the blueprint's defaults.
* **2.3.2. Data Structure (Conceptual)**:
    * `instance_id`: Unique identifier for this placed hull.
    * `blueprint_id`: Reference to the `HullBlueprint` it uses.
    * `portal_connections`: A structure (e.g., `HashMap<u32, PortalConnectionTarget>`) mapping a `portal_id` (a unique identifier for a portal placeholder on its blueprint) to the `instance_id` of the connected hull and the corresponding `portal_id` on the target hull's blueprint. This enables precise portal alignment.
    * `instance_side_configs`: A structure (e.g., `HashMap<u32, SideConfigOverride>`) allowing instance-specific configuration for the side handlers defined in the blueprint.

### 2.4. Side Handlers

* **2.4.1. Concept: Behavior Abstraction**: Instead of sides being rigidly defined as "wall" or "portal," each side of a `HullBlueprint` will be associated with a "Side Handler." This handler is responsible for defining the side's rendering behavior, traversal logic (if any), and interaction properties.
* **2.4.2. Configuration (Blueprint & Instance)**:
    * **Blueprint-Level**: The `BlueprintSide` will specify a default handler type and a default configuration for that handler.
    * **Instance-Level**: The `HullInstance` can override or provide specific parameters for the handlers of its sides (e.g., a portal handler instance gets its target, a wall handler instance gets a specific color).
* **2.4.3. Proposed `SideHandler` Trait (Conceptual)**:
    ```rust
    trait SideHandler {
        // Initialize based on blueprint and instance-specific configurations
        fn setup(&mut self, blueprint_config: &ConfigData, instance_config: &ConfigData);

        // Called by the renderer when processing this side
        // Returns new TraversalStates if this handler opens a view into another space
        fn process_render(
            &self,
            renderer_context: &mut RendererContext, // Access to renderer, current traversal state, etc.
            world_transformed_side_geometry: &Polygon, // Vertices of this side in current view space
            // ... other necessary context like instance_id, blueprint_side_id ...
        ) -> Vec<TraversalState>; // Empty if not a portal-like handler

        // Potentially other methods for updates, interactions, etc.
    }
    ```
* **2.4.4. Example Handler Types**:
    * **`StandardWallHandler`**: Renders an opaque, textured/colored surface. Configuration: texture, color, material properties.
    * **`StandardPortalHandler`**: Represents a traversable opening. Configuration: target `HullInstance ID` and target `portal_id` on the connected hull's blueprint. Defines the relative transformation to the next hull.
    * **`MirrorHandler`**: Renders a reflection of the current hull. Configuration: recursion limits, mirror surface properties. Modifies the accumulated transform with a reflection matrix.
    * **`CameraDisplayHandler`**: Renders a view from a specified in-game camera onto this side's surface. Configuration: source camera ID, render target details, refresh behavior. Involves render-to-texture.
    * **`NonEuclideanPortalHandler`**: A portal whose alignment transformation can include non-uniform scaling or other distortions. Configuration: target connection, specific transformation parameters.
    * **`TransparentWallHandler`**: Renders a transparent surface (e.g., glass). Configuration: tint, opacity, refraction properties.

## 3. Detailed Data Structures (Conceptual Reiteration)

* **`HullBlueprint`**:
    * `id: BlueprintId`
    * `local_vertices: Vec<Point3>`
    * `sides: Vec<BlueprintSide>`
* **`BlueprintSide`**:
    * `vertex_indices: Vec<usize>` (into `local_vertices`)
    * `local_normal: Point3`
    * `handler_type: SideHandlerTypeId` (enum: Wall, Portal, Mirror, etc.)
    * `default_handler_config: Box<dyn HandlerConfig>` (or a similar generic/serialized way to store config specific to the `handler_type`)
    * `local_portal_id: Option<u32>` (if this side can act as a portal, its unique ID within this blueprint)
* **`HullInstance`**:
    * `id: InstanceId`
    * `blueprint_id: BlueprintId`
    * `initial_transform: Option<Mat4>` (Defines transform of the very first hull relative to camera, or if a "master" world origin is used for initial placement of some hulls).
    * `portal_connections: HashMap<u32, PortalConnectionInfo>`
        * Key: `local_portal_id` from its blueprint.
        * Value: `PortalConnectionInfo { target_instance_id: InstanceId, target_portal_id: u32 }`
    * `instance_side_handler_configs: HashMap<u32, Box<dyn HandlerConfig>>` (or similar)
        * Key: `local_portal_id` or a general `blueprint_side_index`.
        * Value: Instance-specific configuration for the handler on that side.
* **`SideHandlerTypeId`**: An enum mapping to concrete handler implementations.
* **`HandlerConfig`**: A trait or enum system for handler-specific configuration data.
* **`Scene`**:
    * `blueprints: HashMap<BlueprintId, HullBlueprint>`
    * `instances: Vec<HullInstance>` (or `HashMap<InstanceId, HullInstance>`)
    * `active_camera_logical_hull: InstanceId` (the hull the camera is currently "in")
    * `active_camera_local_transform: Transform` (camera's pose within its logical hull's blueprint space)

## 4. System Implications

### 4.1. Rendering Pipeline

* **4.1.1. Traversal State & Accumulated Transforms**: The `TraversalState` passed during recursive rendering will need to include the current accumulated transformation matrix (from the initial camera view space to the current hull being rendered) and potentially a reflection state (e.g., determinant sign or bounce count).
* **4.1.2. Portal Alignment Calculation**: For portal-type handlers, the `process_render` method will compute the transformation required to align the connected hull's portal with the current portal, relative to the current accumulated view.
* **4.1.3. Side Handler Invocation**: The renderer, upon encountering a side, will:
    1.  Determine the `HullInstance` and `BlueprintSide`.
    2.  Transform local side geometry using the current accumulated transform.
    3.  Identify the `SideHandlerTypeId` and retrieve its configuration (merged from blueprint default and instance override).
    4.  Invoke the appropriate `SideHandler::process_render` method.

### 4.2. Camera System

* **4.2.1. Localized State**: The main camera's `position` and `orientation` (yaw/pitch) will be stored relative to the local coordinate system of the `HullInstance` it currently, logically occupies.
* **4.2.2. State Transition through Portals**: When the player/camera logically moves through a `StandardPortalHandler` (or similar traversable portal):
    * Its logical `current_hull_instance_id` is updated.
    * Its local `position` and `orientation` are transformed from the old hull's space into the new hull's local space, using the portal alignment transformation. This ensures a seamless transition.

### 4.3. Enabling Advanced Effects ("Exploits")

This architecture is designed to natively support:
* **4.3.1. Mirrors**: A `MirrorHandler` reflects the accumulated transformation and re-queues the current hull for rendering with a modified state.
* **4.3.2. In-Game Cameras & Displays**: A `CameraDisplayHandler` triggers a separate rendering pass (using the same relativistic traversal logic from a different camera's perspective) to a texture, which is then applied to the display surface.
* **4.3.3. Non-Euclidean Geometries & Wormholes**: `NonEuclideanPortalHandler`s can define arbitrary transformations (including scaling) between connected portals, creating distorted or impossible spatial connections.

## 5. Advantages

* **Reduced Data Redundancy**: Hull geometry is defined once per blueprint.
* **Enhanced Design Power**: Complex and varied scenes can be built from a smaller set of reusable blueprints.
* **Native Support for Advanced Visuals**: Mirrors, live camera feeds, and non-Euclidean spaces become more feasible to implement.
* **Extensibility**: New side behaviors can be added by creating new handlers without altering core scene structures.

## 6. Challenges & Future Considerations

* **6.1. Transformation Precision & Management**: Deeply nested portal traversals can accumulate floating-point errors in transformation matrices.
* **6.2. Interaction with Non-Rendering Systems (Physics, AI)**: A purely relativistic coordinate system is challenging for global-space-aware systems. These may require a separate, simplified Euclidean representation or need to become portal-aware themselves. This is a significant complexity for a full game.
* **6.3. Debugging Complexity**: Visualizing and debugging transformations and spatial relationships can be more difficult than with a static global coordinate system.
* **6.4. Handler State Management**: Deciding if side handlers are stateless or stateful and how state is managed.
* **6.5. Authoring Workflow**: Developing tools or clear data formats for defining blueprints, instances, connections, and handler configurations will be crucial.

## 7. Scope for Initial Implementation

While this document outlines a comprehensive system, features can be implemented iteratively. For example:
1.  Implement Hull Blueprints and Instances with a basic `StandardWallHandler` and `StandardPortalHandler` using the relativistic coordinate system.
2.  Then, introduce more complex handlers like `MirrorHandler` or `CameraDisplayHandler` one at a time.

This provides a thorough roadmap for the envisioned system.