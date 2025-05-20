# Engine3 - 3D Portal Rendering and Convex Polygon Intersection

This project is a 3D graphics application built with Rust and `wgpu` that demonstrates portal-based rendering and convex polygon intersection algorithms. It features a navigable 3D scene composed of interconnected rooms (hulls) where visibility is determined by portals, and includes tools for generating, visualizing, and intersecting 2D convex polygons.

## Features

* **3D Portal Rendering:** The core of the application demonstrates a portal rendering technique. The scene is divided into convex regions (hulls), and visibility between these regions is managed through portals (special sides of a hull).
* **Convex Polygon Operations:**
    * **Generation:** Procedurally generates random convex polygons.
    * **Intersection:** Implements the Sutherland-Hodgman algorithm to find the intersection of two convex polygons. This is used in the portal rendering logic for clipping views.
    * **Clipping:** Includes 3D near-plane clipping for polygons in camera space.
* **WGPU for Rendering:** Utilizes the `wgpu` library for graphics rendering, providing a modern, cross-platform graphics API.
* **First-Person Camera:** Implements a camera system with controls for movement (W, A, S, D, Space, Shift/Ctrl) and looking (mouse, arrow keys).
* **Egui for UI:** Integrates `egui` for an in-application GUI, displaying controls and statistics like polygon vertex counts and areas.
* **Scene Definition:** Defines a 3D scene composed of multiple "hulls" (rooms or convex spaces) connected by "portals".
* **Benchmarking:** Includes benchmarks for the convex polygon intersection algorithm using `criterion`.
* **Reference HTML/JS Implementation:** Provides an HTML file with a JavaScript implementation of convex polygon generation and intersection, likely used for reference or comparison during development.

## Core Concepts Demonstrated

* **Portal Culling:** Efficiently rendering complex 3D scenes by only drawing what's visible through a series of portals. The `Renderer` iterates through connected hulls, clipping the view frustum (represented as a 2D screen-space polygon) at each portal.
* **Sutherland-Hodgman Algorithm:** Used for clipping polygons against other polygons, fundamental to the portal rendering (clipping the view against portal boundaries) and 2D intersection logic.
* **3D Graphics Pipeline with WGPU:** Setup of rendering pipelines, buffers (vertex, index, uniform), shaders (WGSL), and handling of window events.
* **Camera Transformations:** Implementing view and projection transformations for a 3D camera.
* **User Input Handling:** Managing keyboard and mouse input for camera control and UI interactions.

## Project Structure

The project is organized into several modules:

* `src/main.rs`: Entry point of the application, sets up the event loop and initializes the `PolygonApp`.
* `src/app.rs`: Contains the main application struct (`PolygonApp`), handles wgpu initialization, event processing, updates, and rendering calls.
* `src/renderer.rs`: Manages the rendering pipeline, scene traversal logic for portal rendering, vertex/index buffer updates, and drawing commands.
* `src/scene.rs`: Defines the structures for `Scene`, `Hull`, and `SceneSide` (which can be walls or portals), and includes a function to create a sample multi-room scene.
* `src/camera.rs`: Implements the `Camera` struct, including methods for transforming points to camera space and projecting them to the screen.
* `src/geometry.rs`: Defines basic geometric primitives like `Point2` and `ConvexPolygon`.
* `src/intersection.rs`: Contains the `ConvexIntersection` struct and the implementation of the Sutherland-Hodgman algorithm for finding the intersection of two convex polygons (`find_intersection_into`).
* `src/generator.rs`: Provides `PolygonGenerator` for creating random convex polygons.
* `src/shader.rs`: Contains the WGSL shader source code for vertex and fragment shaders.
* `src/ui.rs`: Defines the user interface using `egui`, showing controls and statistics.
* `src/vertex.rs`: Defines the `Vertex` struct used for rendering.
* `src/lib.rs`: Library crate root, re-exporting modules like geometry, generator, and intersection.
* `benches/intersection_benchmark.rs`: Performance benchmark for the `find_intersection_into` function.
* `src/reference.html`: An HTML/JavaScript reference implementation for convex polygon intersection visualization.

## Controls

### In-App UI (Egui)
* **"Generate New Polygons" Button**: (Note: This seems to be a leftover from a 2D demo, the 3D scene is fixed)
* **Polygon Statistics Display**: Shows vertex count and area for (again, likely from 2D context, not directly tied to the 3D scene walls).
* **"Enable Animation" Checkbox**: (Functionality might be tied to the 2D polygon demo context)
* **Keyboard Controls List**: Displays available keyboard shortcuts.

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
