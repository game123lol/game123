use std::collections::HashMap;

use hecs::World;
use tetra::{graphics::Texture, math::Vec2, window, Context};

use crate::{Player, Position, Renderable};

pub fn run_render_system(world: &World, ctx: &mut Context, resources: &HashMap<String, Texture>) {
    if let Some((_, (_, Position(cam_pos)))) = world.query::<(&Player, &Position)>().iter().next() {
        for (e, (Renderable(texture, rect), Position(pos))) in
            world.query::<(&Renderable, &Position)>().iter()
        {
            let (w, h) = window::get_size(ctx);
            resources.get(texture).unwrap().draw_region(
                ctx,
                *rect,
                *pos - *cam_pos + Vec2::new(w as f32 / 2., h as f32 / 2.),
            )
        }
    }
}
