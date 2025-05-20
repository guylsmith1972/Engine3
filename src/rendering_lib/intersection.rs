// src/intersection.rs

use crate::geometry::{ConvexPolygon, Point2, MAX_VERTICES};

pub struct ConvexIntersection;

impl ConvexIntersection {
    #[inline(always)]
    fn is_inside(point: &Point2, edge_start: &Point2, edge_end: &Point2) -> bool {
        ((edge_end.x - edge_start.x) * (point.y - edge_start.y) -
         (edge_end.y - edge_start.y) * (point.x - edge_start.x)) >= -1e-5
    }

    fn line_intersection(p1: &Point2, p2: &Point2, clip_edge_p1: &Point2, clip_edge_p2: &Point2) -> Option<Point2> {
        let dx_line = p2.x - p1.x;
        let dy_line = p2.y - p1.y;
        let dx_clip = clip_edge_p2.x - clip_edge_p1.x;
        let dy_clip = clip_edge_p2.y - clip_edge_p1.y;

        let denominator = dy_clip * dx_line - dx_clip * dy_line;

        if denominator.abs() < 1e-10 {
            return None;
        }

        let t = (dx_clip * (p1.y - clip_edge_p1.y) - dy_clip * (p1.x - clip_edge_p1.x)) / denominator;
        
        Some(Point2::new(p1.x + t * dx_line, p1.y + t * dy_line))
    }
    
    fn clip_polygon_by_edge(
        subject_vertices: &[Point2],
        clip_edge_start: &Point2,
        clip_edge_end: &Point2,
        output_buffer: &mut [Point2; MAX_VERTICES],
    ) -> usize {
        if subject_vertices.is_empty() {
            return 0;
        }

        let mut output_count = 0;
        let mut prev_vertex = subject_vertices[subject_vertices.len() - 1];
        
        // This is the version of clip_polygon_by_edge that performed best previously
        // (calling is_inside twice per iteration).
        for &current_vertex in subject_vertices {
            let prev_is_inside = Self::is_inside(&prev_vertex, clip_edge_start, clip_edge_end);
            let current_is_inside = Self::is_inside(&current_vertex, clip_edge_start, clip_edge_end);

            if prev_is_inside && current_is_inside {
                if output_count < MAX_VERTICES {
                    output_buffer[output_count] = current_vertex;
                    output_count += 1;
                } else { break; }
            } else if prev_is_inside && !current_is_inside {
                if let Some(intersection) = Self::line_intersection(&prev_vertex, &current_vertex, clip_edge_start, clip_edge_end) {
                    if output_count < MAX_VERTICES {
                        output_buffer[output_count] = intersection;
                        output_count += 1;
                    } else { break; }
                }
            } else if !prev_is_inside && current_is_inside {
                if let Some(intersection) = Self::line_intersection(&prev_vertex, &current_vertex, clip_edge_start, clip_edge_end) {
                     if output_count < MAX_VERTICES {
                        output_buffer[output_count] = intersection;
                        output_count += 1;
                    } else { break; }
                }
                if output_count < MAX_VERTICES {
                    output_buffer[output_count] = current_vertex;
                    output_count += 1;
                } else { break; }
            }
            prev_vertex = current_vertex;
        }
        output_count
    }

    pub fn find_intersection_into(
        poly1: &ConvexPolygon,
        poly2: &ConvexPolygon,
        result_poly: &mut ConvexPolygon,
    ) {
        let mut buffer_a = [Point2::new(0.0, 0.0); MAX_VERTICES];
        let mut buffer_b = [Point2::new(0.0, 0.0); MAX_VERTICES];
        let mut subject_count;

        subject_count = poly1.count();
        if subject_count == 0 {
            result_poly.set_count(0);
            return;
        }
        // This early exit can be important if poly2 is empty
        if poly2.count() < 3 { // A clipper polygon needs at least 3 vertices to define clip edges
            if subject_count > 0 { // If poly1 has vertices, it's the result (no clipping performed)
                result_poly.copy_vertices_from_slice(poly1.vertices());
            } else {
                result_poly.set_count(0);
            }
            return;
        }

        buffer_a[..subject_count].copy_from_slice(poly1.vertices());

        let mut input_is_buffer_a = true;

        for i in 0..poly2.count() {
            if subject_count == 0 { break; }

            let clip_edge_start = poly2.vertices()[i];
            let clip_edge_end = poly2.vertices()[(i + 1) % poly2.count()];
            
            let (current_subject_slice, output_array_for_clipping): (&[Point2], &mut [Point2; MAX_VERTICES]) = 
                if input_is_buffer_a {
                    (&buffer_a[..subject_count], &mut buffer_b)
                } else {
                    (&buffer_b[..subject_count], &mut buffer_a)
                };
            
            let mut all_inside_this_edge = true;
            for k_idx in 0..subject_count { 
                if !Self::is_inside(&current_subject_slice[k_idx], &clip_edge_start, &clip_edge_end) {
                    all_inside_this_edge = false;
                    break;
                }
            }

            if all_inside_this_edge {
                continue;
            }

            subject_count = Self::clip_polygon_by_edge(
                current_subject_slice,
                &clip_edge_start,
                &clip_edge_end,
                output_array_for_clipping,
            );
            
            input_is_buffer_a = !input_is_buffer_a; 
        }

        let final_vertices_slice = if input_is_buffer_a {
            &buffer_a[..subject_count]
        } else {
            &buffer_b[..subject_count]
        };

        if subject_count > 0 {
            result_poly.copy_vertices_from_slice(final_vertices_slice);
        } else {
            result_poly.set_count(0);
        }
    }
}