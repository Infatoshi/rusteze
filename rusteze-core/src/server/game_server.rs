use crate::entity::entity::EntityKind;
use crate::game::actions::Action;
use crate::game::attack::EntityAttack;
use crate::position::Position;
use crate::server::monster_manager::MonsterManager;
use crate::world::world::World;
use std::sync::{Arc, Mutex};

/// Simplified GameServer for single-player headless use
/// Removed all multiplayer/network concepts
pub struct GameServer {
    /// The full world
    world: Arc<Mutex<World>>,

    /// In charge of handling the entities
    monster_manager: MonsterManager,
}

impl GameServer {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self {
            world: Arc::clone(&world),
            monster_manager: MonsterManager::new(world),
        }
    }

    /// Step the game forward by dt seconds
    pub fn step(&mut self, dt: f32) {
        // Update monsters
        let player_list = Vec::new(); // Empty for now, can be populated if needed
        self.monster_manager.step(dt, &player_list);
    }

    /// Apply an action to the world
    pub fn apply_action(&mut self, action: &Action) {
        self.world.lock().unwrap().apply_action(action);
    }

    /// Get a reference to the world
    pub fn world(&self) -> Arc<Mutex<World>> {
        Arc::clone(&self.world)
    }

    pub fn spawn_monster(&mut self, position: Position) {
        self.monster_manager
            .spawn_new_monster(position, EntityKind::Monster1);
    }

    pub fn on_new_attack(&mut self, attack: EntityAttack) {
        println!("Attack received: {attack:?}");
        let victim = attack.victim_id() as usize;
        // If the victim is a monster, kill it
        self.monster_manager.remove_monster(victim);
    }
}
