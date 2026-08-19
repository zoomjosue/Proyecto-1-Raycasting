use crate::{
    framebuffer::Framebuffer,
    map::{GameMap, CELL_SIZE},
    player::Player,
    texture::TextureAtlas,
};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

pub const FOV: f32 = PI / 3.0;

pub fn render_world(
    framebuffer: &mut Framebuffer,
    map: &GameMap,
    player: &Player,
    textures: &TextureAtlas,
) -> Vec<f32> {
    let horizon = framebuffer.height as i32 / 2;
    for y in 0..horizon {
        let shade = (18 + y * 10 / horizon) as u32;
        framebuffer.horizontal_line(
            y,
            0,
            framebuffer.width as i32 - 1,
            (shade << 16) | (shade << 8) | (shade + 8),
        );
    }
    for y in horizon..framebuffer.height as i32 {
        let shade = (28 + (y - horizon) * 22 / horizon) as u32;
        framebuffer.horizontal_line(
            y,
            0,
            framebuffer.width as i32 - 1,
            (shade << 16) | ((shade / 2) << 8) | (shade / 2),
        );
    }
    let direction = player.forward();
    let plane = Vec2::new(-direction.y, direction.x) * (FOV / 2.0).tan();
    let mut depth_buffer = vec![f32::MAX; framebuffer.width];
    for (screen_x, depth) in depth_buffer.iter_mut().enumerate() {
        let camera_x = 2.0 * screen_x as f32 / framebuffer.width as f32 - 1.0;
        let ray = direction + plane * camera_x;
        let hit = cast_ray(map, player.position, ray);
        let corrected_distance =
            (hit.distance * (ray.x * direction.x + ray.y * direction.y)).max(0.1);
        *depth = corrected_distance;
        let wall_height = (CELL_SIZE * framebuffer.height as f32 / corrected_distance) as i32;
        // Conservamos la posición proyectada completa para calcular la textura.
        // Si la pared está muy cerca, su parte superior queda fuera de la pantalla;
        // usar el `top` recortado como origen hacía que la textura se estirara.
        let projected_top = framebuffer.height as i32 / 2 - wall_height / 2;
        let projected_bottom = framebuffer.height as i32 / 2 + wall_height / 2;
        let top = projected_top.max(0);
        let bottom = projected_bottom.min(framebuffer.height as i32 - 1);
        let light = (1.0 / (1.0 + corrected_distance * 0.005)).max(0.25);
        let side_light = if hit.side == 1 { 0.72 } else { 1.0 };
        for y in top..=bottom {
            let wall_u = hit.wall_x / CELL_SIZE;
            let wall_v = if wall_height > 0 {
                (y - projected_top) as f32 / wall_height as f32
            } else {
                0.0
            };
            let wall_texture = if hit.tile == 'D' {
                textures.sample_door(wall_u, wall_v)
            } else {
                textures.sample(hit.tile, wall_u, wall_v)
            };
            let texture_color = darken(wall_texture, light * side_light);
            framebuffer.set_pixel(screen_x as i32, y, texture_color);
        }
    }
    depth_buffer
}

pub struct RayHit {
    pub distance: f32,
    pub tile: char,
    pub side: i32,
    pub wall_x: f32,
}

pub fn cast_ray(map: &GameMap, origin: Vec2, ray: Vec2) -> RayHit {
    let mut map_x = (origin.x / CELL_SIZE) as i32;
    let mut map_y = (origin.y / CELL_SIZE) as i32;
    let delta_x = if ray.x.abs() < 0.0001 {
        1e30
    } else {
        (CELL_SIZE / ray.x).abs()
    };
    let delta_y = if ray.y.abs() < 0.0001 {
        1e30
    } else {
        (CELL_SIZE / ray.y).abs()
    };
    let (step_x, mut side_x) = if ray.x < 0.0 {
        (-1, (origin.x / CELL_SIZE - map_x as f32) * delta_x)
    } else {
        (1, (map_x as f32 + 1.0 - origin.x / CELL_SIZE) * delta_x)
    };
    let (step_y, mut side_y) = if ray.y < 0.0 {
        (-1, (origin.y / CELL_SIZE - map_y as f32) * delta_y)
    } else {
        (1, (map_y as f32 + 1.0 - origin.y / CELL_SIZE) * delta_y)
    };
    for _ in 0..256 {
        let side = if side_x < side_y {
            side_x += delta_x;
            map_x += step_x;
            0
        } else {
            side_y += delta_y;
            map_y += step_y;
            1
        };
        if map.is_wall(map_x, map_y) {
            let distance = if side == 0 {
                side_x - delta_x
            } else {
                side_y - delta_y
            };
            let hit_position = origin + ray * distance;
            let wall_x = if side == 0 {
                hit_position.y
            } else {
                hit_position.x
            };
            return RayHit {
                distance: distance.max(0.1),
                tile: map.tile_at(map_x, map_y),
                side,
                wall_x,
            };
        }
    }
    RayHit {
        distance: 1.0,
        tile: '#',
        side: 0,
        wall_x: 0.0,
    }
}

fn darken(color: u32, amount: f32) -> u32 {
    let r = (((color >> 16) & 255) as f32 * amount).clamp(0.0, 255.0) as u32;
    let g = (((color >> 8) & 255) as f32 * amount).clamp(0.0, 255.0) as u32;
    let b = ((color & 255) as f32 * amount).clamp(0.0, 255.0) as u32;
    (r << 16) | (g << 8) | b
}
