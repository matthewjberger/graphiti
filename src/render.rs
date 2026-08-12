use crate::geometry::{
    Triangle, disc_triangles, polygon_outline_triangles, ring_triangles, rounded_rect_outline,
    stroke_to_triangles, triangulate_polygon,
};
use crate::scene::{Scene, TextAlign, TextBaseline};
use crate::theme::Rgba;
use nalgebra_glm::{Vec2, Vec3, vec2, vec3};
use nightshade::prelude::{TextProperties, spawn_3d_billboard_text_with_properties};
use nightshade_api::prelude::*;

const PIXELS_TO_WORLD: f32 = 0.01;
const DEPTH_STEP: f32 = 0.0004;
const WORLD_TEXT_BIAS: f32 = 1.7;

struct Batch {
    color: Rgba,
    triangles: Vec<Triangle>,
    depth: f32,
}

pub fn populate_world(world: &mut World, scene: &Scene, supersample: f32) -> Vec<Entity> {
    let unit = PIXELS_TO_WORLD * supersample.max(1.0);
    configure_render_settings(world, scene);
    let mut spawned = spawn_geometry(world, scene, unit);
    spawned.extend(spawn_labels(world, scene, unit, supersample.max(1.0)));
    place_camera(world, scene, supersample);
    spawned
}

pub fn clear_world(world: &mut World, entities: &[Entity]) {
    for entity in entities {
        despawn(world, *entity);
    }
}

fn configure_render_settings(world: &mut World, scene: &Scene) {
    let background = scene.background.unwrap_or(Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 1.0,
    });
    set_background(
        world,
        Background::Color([
            background.red,
            background.green,
            background.blue,
            background.alpha,
        ]),
    );
    show_grid(world, false);
    set_bloom(world, false);
    let settings = world.res_mut::<nightshade::render::config::RenderSettings>();
    settings.unlit_mode = true;
    settings.show_sky = false;
    settings.ssao_enabled = false;
    settings.ssgi_enabled = false;
    settings.ssr_enabled = false;
    settings.taa_enabled = false;
    settings.color_grading.tonemap_algorithm = nightshade::render::config::TonemapAlgorithm::None;
    settings.color_grading.exposure = 1.0;
    settings.color_grading.auto_exposure = false;
    settings.depth_of_field.enabled = false;
}

fn spawn_geometry(world: &mut World, scene: &Scene, unit: f32) -> Vec<Entity> {
    let mut spawned = Vec::new();
    let mut batches: Vec<Batch> = Vec::new();

    for rect in &scene.rects {
        let outline = rounded_rect_outline(rect.position, rect.size, rect.radius, 6);
        if let Some(fill) = rect.paint.fill {
            push_batch(
                &mut batches,
                fill,
                rect.paint.depth,
                triangulate_polygon(&outline),
            );
        }
        if let Some(stroke) = rect.paint.stroke
            && rect.paint.stroke_width > 0.0
        {
            push_batch(
                &mut batches,
                stroke,
                rect.paint.depth + 0.02,
                polygon_outline_triangles(&outline, rect.paint.stroke_width),
            );
        }
    }

    for polygon in &scene.polygons {
        if let Some(fill) = polygon.paint.fill {
            push_batch(
                &mut batches,
                fill,
                polygon.paint.depth,
                triangulate_polygon(&polygon.points),
            );
        }
        if let Some(stroke) = polygon.paint.stroke
            && polygon.paint.stroke_width > 0.0
        {
            push_batch(
                &mut batches,
                stroke,
                polygon.paint.depth + 0.02,
                polygon_outline_triangles(&polygon.points, polygon.paint.stroke_width),
            );
        }
    }

    for circle in &scene.circles {
        let segments = ((circle.radius.max(4.0) * 1.6) as usize).clamp(16, 96);
        if let Some(fill) = circle.paint.fill {
            push_batch(
                &mut batches,
                fill,
                circle.paint.depth,
                disc_triangles(circle.center, circle.radius, segments),
            );
        }
        if let Some(stroke) = circle.paint.stroke
            && circle.paint.stroke_width > 0.0
        {
            push_batch(
                &mut batches,
                stroke,
                circle.paint.depth + 0.02,
                ring_triangles(
                    circle.center,
                    circle.radius,
                    circle.paint.stroke_width,
                    segments,
                ),
            );
        }
    }

    for stroke in &scene.strokes {
        push_batch(
            &mut batches,
            stroke.color,
            stroke.depth,
            stroke_to_triangles(&stroke.points, stroke.width, false, stroke.dash),
        );
    }

    for (index, batch) in batches.iter().enumerate() {
        if batch.triangles.is_empty() {
            continue;
        }
        let mut vertices = Vec::with_capacity(batch.triangles.len() * 3);
        let mut indices = Vec::with_capacity(batch.triangles.len() * 3);
        let z = batch.depth * DEPTH_STEP;
        for triangle in &batch.triangles {
            for corner in [0usize, 2, 1] {
                let world_point = to_world(triangle[corner], scene, z, unit);
                vertices.push((
                    [world_point.x, world_point.y, world_point.z],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0],
                ));
                indices.push(indices.len() as u32);
            }
        }
        let name = format!("graphiti::batch::{index}");
        let entity = spawn_custom_mesh(world, &name, &vertices, &indices, vec3(0.0, 0.0, 0.0));
        set_color(
            world,
            entity,
            [
                batch.color.red,
                batch.color.green,
                batch.color.blue,
                batch.color.alpha,
            ],
        );
        spawned.push(entity);
    }
    spawned
}

fn spawn_labels(world: &mut World, scene: &Scene, unit: f32, supersample: f32) -> Vec<Entity> {
    let mut spawned = Vec::with_capacity(scene.labels.len());
    for label in &scene.labels {
        let alignment = match label.style.align {
            TextAlign::Left => TextAlignment::Left,
            TextAlign::Center => TextAlignment::Center,
            TextAlign::Right => TextAlignment::Right,
        };
        let size = label.style.size;
        let centering = size * WORLD_TEXT_BIAS
            + match label.style.baseline {
                TextBaseline::Top => size * 0.5,
                TextBaseline::Middle => 0.0,
                TextBaseline::Bottom => -size * 0.5,
            };
        let anchored = vec2(label.position.x, label.position.y + centering);
        let position = to_world(anchored, scene, label.style.depth * DEPTH_STEP, unit);
        let entity = spawn_3d_billboard_text_with_properties(
            world,
            &label.text,
            position,
            TextProperties {
                font_size: size * supersample,
                color: Vec4::new(
                    label.style.color.red,
                    label.style.color.green,
                    label.style.color.blue,
                    label.style.color.alpha,
                ),
                alignment,
                vertical_alignment: VerticalAlignment::Middle,
                line_height: 1.0,
                font_kind: if label.style.monospace {
                    nightshade::text::text_data::FontKind::Mono
                } else {
                    nightshade::text::text_data::FontKind::Default
                },
                ..TextProperties::default()
            },
        );
        spawned.push(entity);
    }
    spawned
}

fn place_camera(world: &mut World, scene: &Scene, supersample: f32) {
    fixed_camera(world, vec3(0.0, 0.0, 20.0), vec3(0.0, 0.0, 0.0));
    fit_camera(world, scene.size, supersample);
}

pub fn fit_camera(world: &mut World, scene_size: Vec2, supersample: f32) {
    let unit = PIXELS_TO_WORLD * supersample.max(1.0);
    let aspect = world
        .res::<nightshade::platform::window::Window>()
        .cached_viewport_size
        .map(|(width, height)| width as f32 / height.max(1) as f32)
        .unwrap_or(scene_size.x / scene_size.y.max(1.0));
    let half_height = (scene_size.y * 0.5).max(scene_size.x * 0.5 / aspect.max(0.01));
    set_orthographic(world, half_height * unit);
}

fn push_batch(batches: &mut Vec<Batch>, color: Rgba, depth: f32, triangles: Vec<Triangle>) {
    if triangles.is_empty() {
        return;
    }
    if let Some(batch) = batches
        .iter_mut()
        .find(|batch| batch.color == color && (batch.depth - depth).abs() < 0.001)
    {
        batch.triangles.extend(triangles);
        return;
    }
    batches.push(Batch {
        color,
        triangles,
        depth,
    });
}

fn to_world(point: Vec2, scene: &Scene, z: f32, unit: f32) -> Vec3 {
    vec3(
        (point.x - scene.size.x * 0.5) * unit,
        (scene.size.y * 0.5 - point.y) * unit,
        z,
    )
}
