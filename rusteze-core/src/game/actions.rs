use crate::vector::Vector3;
use crate::world::block_kind::Block;
use serde::{Deserialize, Serialize};

/// Player movement and camera input for controlling the agent.
/// 
/// This struct represents all possible player inputs that can be sent to the environment.
/// All fields are optional and default to `false` or `None` if not provided.
/// 
/// # Example
/// ```rust
/// use rusteze_core::game::actions::PlayerInput;
/// 
/// let input = PlayerInput {
///     camera: Some([15.0, 0.0]),  // Turn right 15 degrees
///     forward: true,
///     attack: false,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct PlayerInput {
    /// Camera rotation: [horizontal (yaw), vertical (pitch)] in degrees
    #[serde(default)]
    pub camera: Option<[f32; 2]>,

    /// Forward movement
    #[serde(default)]
    pub forward: bool,

    /// Backward movement
    #[serde(default)]
    pub back: bool,

    /// Left strafe
    #[serde(default)]
    pub left: bool,

    /// Right strafe
    #[serde(default)]
    pub right: bool,

    /// Jump
    #[serde(default)]
    pub jump: bool,

    /// Attack/break block
    #[serde(default)]
    pub attack: bool,
}

/// An action is something that will alter the world and/or player state.
/// 
/// Actions can be:
/// - `Noop`: Do nothing
/// - `Destroy`: Remove a block at a specific position
/// - `Add`: Place a block at a specific position
/// - `PlayerInput`: Control the player's movement and camera
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    /// No operation - useful for testing/default actions
    Noop {},

    /// Destroys a cube of the world
    Destroy {
        at: Vector3,
    },

    // Adds a cube
    Add {
        at: Vector3,
        block: Block,
    },

    /// Player movement and input actions
    PlayerInput {
        input: PlayerInput,
    },
}

impl Default for Action {
    fn default() -> Self {
        Action::Noop {}
    }
}

impl Action {
    pub fn to_bytes(&self) -> Vec<u8> {
        let as_json = serde_json::to_string(self).unwrap();
        as_json.into_bytes()
    }

    pub fn from_str(text: &str) -> Self {
        serde_json::from_str(text).unwrap_or_default()
    }

    /// Create an action from a PlayerInput
    pub fn from_player_input(input: PlayerInput) -> Self {
        Action::PlayerInput { input }
    }

    /// Get player input if this action contains it
    pub fn player_input(&self) -> Option<&PlayerInput> {
        match self {
            Action::PlayerInput { input } => Some(input),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vector::Vector3;
    use crate::world::block_kind::Block;
    use crate::world::cube::Cube;

    #[test]
    fn test_computation_of_new_cube_position() {
        let cube = Cube::new([0., 0., 0.], Block::COBBELSTONE, 0);

        assert_eq!(
            Vector3::new(1., 0., 0.),
            cube.position_to_add_new_cube(Vector3::new(3., 0.5, 0.5), Vector3::unit_x().opposite())
                .unwrap()
        );

        assert_eq!(
            Vector3::new(0., 0., 1.),
            cube.position_to_add_new_cube(
                Vector3::new(0.5, 0.5, 3.5),
                Vector3::unit_z().opposite()
            )
            .unwrap()
        );
    }
}
