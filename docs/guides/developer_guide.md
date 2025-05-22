# Engine3 - Developer's Guide: Core Rendering Concepts

## Introduction

Welcome to Engine3! This guide is intended for developers working on or extending the engine. Engine3 employs some specific architectural choices, particularly regarding its scene structure and rendering pipeline, that differ from more traditional 3D engines. Understanding these core concepts is crucial for effective development.

This document focuses on clarifying these unique characteristics, especially those that have been sources of confusion, to ensure a common understanding of expected behavior.

## 1. Scene Architecture: Blueprints and Relativistic Hull-Space

The fundamental shift in Engine3 is its **blueprint-based scene definition** and **relativistic hull-space coordinate system**, as detailed in `docs/planning/blueprints.md`.

* **HullBlueprints:** Define the intrinsic, reusable geometry of a convex spatial region (a "hull" or "room") in its own **local blueprint space**. Vertices are relative to this local origin.
* **HullInstances:** Represent specific occurrences of these blueprints in the "world." They do **not** typically have a fixed global world-space transform. Instead, their position and orientation are determined *dynamically* through portal connections relative to the hull the camera currently occupies.
* **No Global World Space (for Hulls):** The engine avoids a single, static world coordinate system for all hull geometry. The "world" is rendered relativistically from the camera's current hull outwards.

## 2. The Camera: Inside a Hull

* **Camera's Location:** The main camera always resides *inside* a specific `HullInstance` (identified by `Scene::active_camera_instance_id`).
* **Camera's Pose:** The camera's position and orientation are defined by `Scene::active_camera_local_transform`, which is a `Mat4` representing the camera's pose (transform from camera local space to its host hull's blueprint space).
* **Initial View Matrix:** The primary view matrix (`camera_view_from_host_hull` in the renderer) is the inverse of `active_camera_local_transform`. It transforms points from the camera's host hull blueprint space into camera view space.

## 3. Hull Geometry: Rendering Interiors and Normal Conventions

This is a critical area where Engine3's conventions for rendering *interior spaces* must be understood:

* **Hollow Hulls, Interior View:** The engine is designed to render the *interiors* of these hollow, convex hulls. The camera is always inside one.
* **Blueprint Side Normals Point INWARD:** As a strict rule, the `local_normal` for every `BlueprintSide` (whether an opaque wall or a portal face) **must point TOWARD THE INTERIOR of its own blueprint.**
    * Example for a cuboid blueprint centered at `(0,0,0)`:
        * The +Z face (at `z = max_z`): Its interior-pointing normal is `(0,0,-1)`.
        * The -Z face (at `z = min_z`): Its interior-pointing normal is `(0,0,1)`.
        * The +X face: Its interior-pointing normal is `(-1,0,0)`.
        * And so on for all faces.

## 4. Visibility and Back-Face Culling for Interior Surfaces

Given that normals point inward and the camera is inside:

* **"Visible" Interior Face:** An interior face of the current hull is considered visible (or "front-facing" from the camera's perspective within the hull) if its interior-pointing normal, after transformation to camera view space (`N_view`), generally points **along the camera's viewing direction (away from the camera's origin in view space)**.
* **Camera View Space Convention:** The camera is at the origin `(0,0,0)` looking down its local -Z axis.
* **Culling Rule:**
    * A side (wall or portal) is processed/rendered/traversed if its transformed interior normal `N_view` has `N_view.z > +epsilon` (where `epsilon` is a small positive threshold, e.g., `1e-3`). This means the inward-pointing normal is directed away from the camera's eye and generally aligns with its line of sight into the scene.
    * A side is culled if `N_view.z <= +epsilon` (i.e., its inward normal points back towards the camera's eye or is edge-on).

* **Application to `StandardPortalHandler`:** This culling rule is applied in `StandardPortalHandler` *before* deciding to traverse a portal. If a portal face's transformed interior normal does not satisfy `N_view.z > epsilon`, the portal is culled for traversal from that viewpoint.

## 5. Portal Traversal and Transform Accumulation

* **Fixed Camera, Transformed World:** When rendering through portals, the player's actual camera position (`active_camera_local_transform`) **does not change** for that frame's rendering.
* **`portal_alignment_transform`:** When a portal (P1 on Hull A) leads to another portal (P2 on Hull B), the `StandardPortalHandler` calculates a `portal_alignment_transform`. This matrix transforms coordinates from **Hull B's blueprint space** into **Hull A's blueprint space**, aligning P2 with P1 as if they form a continuous opening.
    * For simple opposing faces (e.g., +Z of Hull A to -Z of Hull B, where blueprints are similarly oriented), this transform is primarily a **translation** to bring the origins of the blueprints into the correct relative positions so the faces meet. It does *not* typically involve a 180-degree rotation if the blueprint faces are already geometrically opposed.
* **`accumulated_transform` (`T_currentBP_to_hostBP`):**
    * For the first hull (camera's host), this is `Mat4::identity()`.
    * When traversing from Hull A to Hull B, the new accumulated transform for Hull B is `T_B_to_hostBP = (T_A_to_hostBP) * (portal_alignment_transform_B_to_A)`.
    * This chain ensures all geometry from any traversed hull is correctly brought into the coordinate system of the camera's original host hull.
* **Final Vertex Transformation:** A vertex `v_local` in the currently processed hull is ultimately transformed to view space by:
    `v_view = camera_view_from_host_hull * accumulated_transform * v_local`.

## 6. Reconciling the "Open Doorway" (e.g., Office-Kitchen)

Consider Room 1 (Office) connecting to Room 2 (Kitchen) via a shared open doorway. P1 is R1's side of the doorway, P2 is R2's side. Both R1 and R2 have their portals defined in `demo_scene.rs` with `HandlerConfig::StandardPortal` pointing to each other. Normals are INWARD.

* **Viewing from Room 1 into Room 2:**
    1.  **Portal P1 (R1's +Z face):** `N1_local_interior = (0,0,-1)`. The camera in R1 is set up with a 180-degree yaw to look at this face.
        * The rotation component of `camera_view_from_host_hull` is `RotY(PI)`.
        * `N1_view = RotY(PI) * (0,0,-1) = (0,0,1)`. So, `N1_view.z = 1`.
        * Culling check: `1 > epsilon` is **TRUE**. **P1 is TRAVERSED.** (This matches spec point 5 & 7).
    2.  **Rendering Room 2 (viewed through P1):**
        * The `portal_alignment_transform` for R1->R2 is now a translation only (e.g., `Translate(0,0,3.0)`). Its rotation part is Identity.
        * The `accumulated_transform` for Room 2 has a rotation part of `Identity`.
        * The total rotation from R2_Blueprint_Space to View_Space is `RotOnly(camera_view_from_host_hull) * RotOnly(accum_for_R2) = RotY(PI) * Identity = RotY(PI)`.
    3.  **Considering Portal P2 (R2's -Z face) for back-traversal:**
        * `N2_local_interior = (0,0,1)`.
        * `N2_view = RotY(PI) * N2_local_interior = RotY(PI) * (0,0,1) = (0,0,-1)`.
        * `N2_view.z = -1`.
        * Culling check: `-1 > epsilon` is **FALSE**.
        * **Result for P2: CULLED.** (This matches spec point 6 & 8).

This sequence, with INWARD blueprint normals, TRANSLATION-ONLY portal alignment for opposed faces, and the culling rule `TRAVERSE if N_view.z > epsilon`, correctly implements the "office-kitchen" scenario where you see into the kitchen, but the kitchen's side of the doorway does not cause a traversal back into the office from that same viewpoint.

## Summary of Key Conventions for This Engine

* **Coordinates:** Relativistic hull-space. Blueprints are in their own local space.
* **Camera:** Always inside a hull; its pose is relative to that hull's blueprint.
* **Hull Side Normals:** **Strictly point towards the interior of their blueprint.**
* **Visibility/Traversal Culling (for interior normals):** A side is processed if its transformed interior normal `N_view` points generally along the camera's view direction (away from the camera's origin in view space), i.e., `N_view.z > +epsilon` when the camera looks down its local -Z.
* **Portal Alignment:** Primarily translational for already opposed faces. Rotations are only included if the blueprints themselves need reorientation relative to each other for their portals to align.

Adhering to these conventions is essential for predictable rendering behavior in Engine3.

---