// Primitives modules
pub mod camera;
pub mod color;
pub mod face;
pub mod math;
pub mod matrix;
pub mod position;
pub mod vector;

// TODO: These have to be moved away from primtiives
//       Because I don't want to have to depend on glium, but I am blocked by the orphan rule.
// Problem Recap
// graphics crate: Defines traits like Renderer. Should not depend on backend details.
// graphics_glium crate: Implements Renderer using glium. It needs types that implement glium::Vertex.
// Orphan rule issue: You can't implement a foreign trait (glium::Vertex) for a foreign type
// (your graphics-defined vertex structs) inside graphics_glium without making graphics depend on glium.
pub mod opengl {
    pub mod cube;
    pub mod cube_instance;
    pub mod entity;
    pub mod font;
    pub mod rectangle;
}

// Model modules
pub mod entity {
    pub mod chaser;
    pub mod entity;
    pub mod entity_manager;
    pub mod humanoid;
    pub mod monster;
    pub mod walker_in_circle;
}

pub mod server {
    pub mod game_server;
    pub mod monster_manager;
    pub mod server_state;
    pub mod server_update;
    pub mod world_dispatcher;
}

pub mod world {
    pub mod block_kind;
    pub mod chunk;
    pub mod cube;
    pub mod cubes_to_draw;
    pub mod generation;
    pub mod world;
    pub mod world_serializer;
}

pub mod game {
    pub mod actions;
    pub mod attack;
    pub mod crafting;
    pub mod health;
    pub mod input;
    pub mod player;
    pub mod player_items;
}

pub mod collision {
    pub mod aabb;
    pub mod collidable;
}

pub mod args;

pub mod env;
pub mod events;
pub mod headless_renderer;
pub mod multi_env;
pub mod reward_manager;
pub mod shaders;

#[cfg(test)]
mod lib_tests;

#[cfg(feature = "extension-module")]
use crate::env::RustezeEnv;
#[cfg(feature = "extension-module")]
use crate::multi_env::MultiRustezeEnv;
#[cfg(feature = "extension-module")]
use pyo3::prelude::*;

#[cfg(feature = "extension-module")]
#[pymodule]
fn rusteze_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustezeEnv>()?;
    m.add_class::<MultiRustezeEnv>()?;
    Ok(())
}
