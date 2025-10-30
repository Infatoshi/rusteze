use crate::events::GameEvent;
use crate::game::actions::Action;
use crate::game::player::Player;
use crate::headless_renderer::HeadlessRenderer;
use crate::position::Position;
use crate::reward_manager::RewardManager;
use crate::server::game_server::GameServer;
use crate::world::chunk::CHUNK_FLOOR;
use crate::world::generation::world_generator::WorldGenerator;
use crate::world::world::World;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(feature = "extension-module")]
use numpy::{IntoPyArray, PyArray, PyArrayMethods};
#[cfg(feature = "extension-module")]
use pyo3::prelude::*;

/// Main environment struct for the Rusteze headless game engine.
/// This is the primary interface for interacting with the Rusteze environment
/// from Rust code. It encapsulates the game world, player, renderer, and reward system.
/// # Example
/// ```no_run
/// use rusteze_core::env::RustezeEnv;
/// 
/// let mut env = RustezeEnv::new(42);
/// let obs = env.reset_internal();
/// let (obs, reward, done) = env.step_internal(rusteze_core::game::actions::Action::default());
/// ```
#[cfg_attr(feature = "extension-module", pyclass)]
pub struct RustezeEnv {
    world: Arc<Mutex<World>>,
    player: Player,
    game_server: GameServer,
    renderer: HeadlessRenderer,
    reward_manager: RewardManager,
    seed: u64,
}

impl RustezeEnv {
    /// Create a new Rusteze environment with the given seed.
    /// 
    /// # Arguments
    /// * `seed` - Random seed for world generation. Same seed produces same world.
    /// 
    /// # Returns
    /// A new `RustezeEnv` instance ready to use.
    pub fn new(seed: u64) -> Self {
        // Initialize world generator with seed
        let world = WorldGenerator::create_new_random_world(5);
        let world = Arc::new(Mutex::new(world));

        // Create game server
        let game_server = GameServer::new(Arc::clone(&world));

        // Create player
        let mut player = Player::new();
        // Spawn higher to avoid collision issues
        let spawn_pos = Position::spawn_position(CHUNK_FLOOR as f32 + 15.);
        player.set_position(spawn_pos);

        // Create headless renderer (640x360 as specified in requirements)
        let renderer = HeadlessRenderer::new(640, 360);

        // Create reward manager
        let reward_manager = RewardManager::new();

        Self {
            world,
            player,
            game_server,
            renderer,
            reward_manager,
            seed,
        }
    }

    /// Reset the environment and return the initial observation.
    /// 
    /// This regenerates the world with the same seed and resets the player position.
    /// 
    /// # Returns
    /// A `Vec<u8>` containing RGB pixel data (width * height * 3 bytes).
    pub fn reset_internal(&mut self) -> Vec<u8> {
        // Regenerate world with same seed
        let world = WorldGenerator::create_new_random_world(5);
        *self.world.lock().unwrap() = world;

        // Reset game server
        self.game_server = GameServer::new(Arc::clone(&self.world));

        // Reset player position (spawn higher to avoid collisions)
        let spawn_pos = Position::spawn_position(CHUNK_FLOOR as f32 + 15.);
        self.player.set_position(spawn_pos);

        // Render initial frame
        let world = self.world.lock().unwrap();
        self.renderer.render(&world, &self.player)
    }

    /// Step the environment forward with the given action.
    /// 
    /// This processes the action, updates the game state, renders a new frame,
    /// and calculates rewards based on events that occurred.
    /// 
    /// # Arguments
    /// * `action` - The action to perform (can be PlayerInput, Destroy, Add, or Noop).
    /// 
    /// # Returns
    /// A tuple containing:
    /// - `observation`: RGB pixel data (width * height * 3 bytes).
    /// - `reward`: Reward value based on events (block breaking, movement, etc.).
    /// - `done`: Whether the episode is finished (always false for now).
    pub fn step_internal(&mut self, action: Action) -> (Vec<u8>, f32, bool) {
        // Track player position before step for movement calculation
        let player_pos_before = self.player.position().pos();

        // Collect events during this step
        let mut events = Vec::new();

        // Handle player input from action
        if let Some(input) = action.player_input() {
            // Handle camera movement
            if let Some([h, v]) = input.camera {
                let sensitivity = 0.01; // Convert degrees to radians approximately
                self.player.mousemove(h, v, sensitivity);
            }

            // Handle movement keys
            use crate::game::input::MotionState;
            self.player.toggle_state(MotionState::Up, input.forward);
            self.player.toggle_state(MotionState::Down, input.back);
            self.player.toggle_state(MotionState::Left, input.left);
            self.player.toggle_state(MotionState::Right, input.right);
            self.player.toggle_state(MotionState::Jump, input.jump);

            // Handle attack (break block) - will be done after player update
            if input.attack {
                self.player.toggle_state(MotionState::LeftClick, true);
            } else {
                self.player.toggle_state(MotionState::LeftClick, false);
            }
        }

        // Step game simulation (fixed timestep)
        let dt = 1.0 / 60.0; // 60 FPS
        self.game_server.step(dt);

        // Update player (this will recompute selected cube via raycasting)
        let mut world = self.world.lock().unwrap();
        self.player.step(Duration::from_secs_f32(dt), &world);

        // Handle block breaking after player update (so raycasting is current)
        if let Some(input) = action.player_input() {
            if input.attack {
                // Use player's selected cube (raycasting result) to break block
                if let Some(selected_cube) = self.player.selected_cube() {
                    let cube_pos = selected_cube.position().clone();
                    let block_type = *selected_cube.block();
                    drop(world);
                    // Create a Destroy action to break the block
                    let destroy_action = Action::Destroy { at: cube_pos };
                    self.game_server.apply_action(&destroy_action);
                    // Emit event for block breaking
                    events.push(GameEvent::BlockBroken { block_type });
                    world = self.world.lock().unwrap();
                }
            }
        }

        // Apply action to game server (for world modifications like Destroy, Add)
        match &action {
            Action::Destroy { .. } => {
                // Direct Destroy action - try to get block type from world
                if let Action::Destroy { at } = &action {
                    if let Some(block_type) = world.block_at(at) {
                        events.push(GameEvent::BlockBroken { block_type });
                    }
                }
                self.game_server.apply_action(&action);
            }
            Action::Add { at: _at, block } => {
                // Emit event for block placing
                events.push(GameEvent::BlockPlaced { block_type: *block });
                self.game_server.apply_action(&action);
            }
            _ => {}
        }
        drop(world);

        // Calculate player movement distance
        let player_pos_after = self.player.position().pos();
        let movement_distance = player_pos_before.distance_to(&player_pos_after);
        if movement_distance > 0.0 {
            events.push(GameEvent::PlayerMoved {
                distance: movement_distance,
            });
        }

        // Render new frame
        let world = self.world.lock().unwrap();
        let observation = self.renderer.render(&world, &self.player);
        drop(world);

        // Calculate reward from events
        let reward = self.reward_manager.calculate_reward(&events);

        // Check if done (never done for now)
        let done = false;

        (observation, reward, done)
    }
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl RustezeEnv {
    #[new]
    fn py_new(seed: u64) -> Self {
        Self::new(seed)
    }

    fn reset(&mut self, py: Python) -> Py<PyArray<u8, numpy::Ix3>> {
        let pixels = self.reset_internal();
        // Convert Vec<u8> to numpy array (height, width, 3)
        // Create array directly with correct shape
        let arr = PyArray::from_vec_bound(py, pixels)
            .reshape([360, 640, 3])
            .unwrap();
        arr.into()
    }

    fn step(
        &mut self,
        action: Option<PyObject>,
        py: Python,
    ) -> PyResult<(Py<PyArray<u8, numpy::Ix3>>, f32, bool)> {
        // Default to Noop if no action provided
        let action_rust: Action = if let Some(action_obj) = action {
            // Try to parse from JSON string first
            if let Ok(json_str) = action_obj.extract::<String>(py) {
                Action::from_str(&json_str)
            } else {
                // Try to parse as dict with PlayerInput fields
                if let Ok(dict) = action_obj.downcast::<pyo3::types::PyDict>(py) {
                    let mut input = crate::game::actions::PlayerInput::default();

                    // Parse camera
                    if let Ok(camera) = dict.get_item("camera") {
                        if let Ok(camera_list) =
                            camera.and_then(|c| c.downcast::<pyo3::types::PyList>())
                        {
                            if camera_list.len() == 2 {
                                if let (Ok(h), Ok(v)) = (
                                    camera_list.get_item(0).and_then(|x| x.extract::<f32>()),
                                    camera_list.get_item(1).and_then(|x| x.extract::<f32>()),
                                ) {
                                    input.camera = Some([h, v]);
                                }
                            }
                        }
                    }

                    // Parse movement keys
                    if let Ok(val) = dict.get_item("forward").and_then(|x| x.extract::<bool>()) {
                        input.forward = val;
                    }
                    if let Ok(val) = dict.get_item("back").and_then(|x| x.extract::<bool>()) {
                        input.back = val;
                    }
                    if let Ok(val) = dict.get_item("left").and_then(|x| x.extract::<bool>()) {
                        input.left = val;
                    }
                    if let Ok(val) = dict.get_item("right").and_then(|x| x.extract::<bool>()) {
                        input.right = val;
                    }
                    if let Ok(val) = dict.get_item("jump").and_then(|x| x.extract::<bool>()) {
                        input.jump = val;
                    }
                    if let Ok(val) = dict.get_item("attack").and_then(|x| x.extract::<bool>()) {
                        input.attack = val;
                    }

                    Action::from_player_input(input)
                } else {
                    Action::Noop {}
                }
            }
        } else {
            Action::Noop {}
        };

        let (obs, reward, done) = self.step_internal(action_rust);
        let arr = PyArray::from_vec_bound(py, obs)
            .reshape([360, 640, 3])
            .unwrap();
        Ok((arr.into(), reward, done))
    }

    fn width(&self) -> usize {
        640
    }

    fn height(&self) -> usize {
        360
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::actions::Action;

    #[test]
    fn test_env_step() {
        let mut env = RustezeEnv::new(42);
        let (obs, reward, done) = env.step_internal(Action::default());
        assert_eq!(obs.len(), 640 * 360 * 3);
        assert!(!done);
    }
}
