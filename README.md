# Engine3 - 3D Portal Rendering and Convex Polygon Intersection

This project is a 3D graphics application built with Rust and `wgpu` that demonstrates portal-based rendering and convex polygon intersection algorithms. It features a navigable 3D scene composed of interconnected rooms (hulls) where visibility is determined by portals. The underlying convex polygon intersection algorithms are also used for tasks like view frustum clipping.

## Features

* **3D Portal Rendering:** The core of the application demonstrates a portal rendering technique. The scene is divided into convex regions (hulls), and visibility between these regions is managed through portals (special sides of a hull).
* **Convex Polygon Operations:**
    * **Intersection:** Implements the Sutherland-Hodgman algorithm to find the intersection of two convex polygons. This is used in the portal rendering logic for clipping views.
    * **Clipping:** Includes 3D near-plane clipping for polygons in camera space.
* **WGPU for Rendering:** Utilizes the `wgpu` library for graphics rendering, providing a modern, cross-platform graphics API.
* **First-Person Camera:** Implements a camera system with controls for movement (W, A, S, D, Space, Shift/Ctrl) and looking (mouse, arrow keys).
* **Egui for UI:** Integrates `egui` for an in-application GUI, displaying controls and information.
* **Scene Definition:** Defines a 3D scene composed of multiple "hulls" (rooms or convex spaces) connected by "portals".
* **Benchmarking:** Includes benchmarks for the convex polygon intersection algorithm using `criterion` (see `benches/intersection_benchmark.rs`).
* **Reference HTML/JS Implementation:** Provides an HTML file (`src/reference.html`) with a JavaScript implementation of 2D convex polygon generation and intersection, used for reference or comparison during development.

## Core Concepts Demonstrated

* **Portal Culling:** Efficiently rendering complex 3D scenes by only drawing what's visible through a series of portals. The `Renderer` iterates through connected hulls, clipping the view frustum (represented as a 2D screen-space polygon) at each portal.
* **Sutherland-Hodgman Algorithm:** Used for clipping polygons against other polygons, fundamental to the portal rendering (clipping the view against portal boundaries).
* **3D Graphics Pipeline with WGPU:** Setup of rendering pipelines, buffers (vertex, index, uniform), shaders (WGSL), and handling of window events.
* **Camera Transformations:** Implementing view and projection transformations for a 3D camera.
* **User Input Handling:** Managing keyboard and mouse input for camera control and UI interactions.

## Project Structure

The project is organized into several modules and libraries:

* `src/main.rs`: Entry point of the application, sets up the event loop and initializes the `PolygonApp`.
* `src/app.rs`: Contains the main application struct (`PolygonApp`), handles wgpu initialization, event processing via `CameraController`, updates, and rendering calls.
* `src/ui.rs`: Defines the user interface using `egui`, showing controls and information.
* `src/demo_scene.rs`: Contains logic to create a sample multi-room 3D scene using types from `engine_lib`.

* **`src/engine_lib/`**: A library for core engine logic, excluding direct rendering.
    * `lib.rs`: Exports modules of the `engine_lib`.
    * `camera.rs`: Implements the `Camera` struct, including methods for transforming points and projection, but relies on `rendering_lib` for `Point2`.
    * `controller.rs`: Implements `CameraController` for handling user input (keyboard/mouse) for camera control.
    * `scene_types.rs`: Defines the structures for `Scene`, `Hull`, `SceneSide`, `Point3`, and `TraversalState`. It relies on `rendering_lib` for `ConvexPolygon`.

* **`src/rendering_lib/`**: A library dedicated to rendering logic and 2D geometry operations.
    * `lib.rs`: Exports modules of the `rendering_lib`.
    * `renderer.rs`: Manages the WGPU rendering pipeline, scene traversal logic for portal rendering (using types from `engine_lib`), vertex/index buffer updates, and drawing commands.
    * `geometry.rs`: Defines basic 2D geometric primitives like `Point2` and `ConvexPolygon`, and `MAX_VERTICES`.
    * `intersection.rs`: Contains `ConvexIntersection` and the Sutherland-Hodgman algorithm for 2D convex polygon intersection.
    * `shader.rs`: Contains the WGSL shader source code.
    * `vertex.rs`: Defines the `Vertex` struct used for rendering.

* `benches/`: Contains criterion benchmarks.
    * `intersection_benchmark.rs`: Performance benchmark for the polygon intersection function.
    * `generator.rs`: Utility for generating random convex polygons for benchmarks.

* `references/sutherland_hodgman_intersection.html`: An HTML/JavaScript reference implementation for 2D convex polygon intersection visualization. (Assuming this path is correct, previously it was `src/reference.html`)

## Controls

### In-App UI (Egui)
* Displays keyboard and mouse controls.

### Keyboard
* **W, S, A, D**: Move camera forward, backward, left, and right.
* **Space**: Move camera up.
* **Left Shift / Left Control**: Move camera down.
* **ArrowLeft, ArrowRight**: Rotate camera yaw (look left/right).
* **ArrowUp, ArrowDown**: Rotate camera pitch (look up/down).
* **Escape**: Grab/Ungrab mouse cursor for camera look control.

### Mouse
* **Motion (when cursor grabbed)**: Controls camera yaw and pitch.
* **Left Click (when cursor not grabbed and window focused)**: Grabs the cursor for camera control.

## Getting Started

### Prerequisites
* Rust programming language and Cargo package manager.

### Building and Running
1.  Clone the repository.
2.  Navigate to the project directory.
3.  Run the application:
    ```bash
    cargo run
    ```

### Running Benchmarks
To run the intersection algorithm benchmarks:
```bash
cargo bench