use crate::events::GameEvent;
use crate::world::block_kind::Block;
use std::collections::HashMap;

/// Configuration for reward values
#[derive(Debug, Clone)]
pub struct RewardConfig {
    /// Reward for breaking each block type
    pub break_rewards: HashMap<Block, f32>,

    /// Reward per unit distance moved
    pub movement_reward: f32,

    /// Reward for placing each block type
    pub place_rewards: HashMap<Block, f32>,
}

impl Default for RewardConfig {
    fn default() -> Self {
        let mut break_rewards = HashMap::new();
        let mut place_rewards = HashMap::new();

        // Set default rewards for breaking blocks
        break_rewards.insert(Block::GRASS, 1.0);
        break_rewards.insert(Block::DIRT, 1.0);
        break_rewards.insert(Block::STONE, 5.0);
        break_rewards.insert(Block::COBBELSTONE, 2.0);
        break_rewards.insert(Block::OAKLOG, 3.0);
        break_rewards.insert(Block::OAKLEAVES, 1.0);
        break_rewards.insert(Block::SAND, 1.0);
        break_rewards.insert(Block::WATER, 0.0);
        break_rewards.insert(Block::SWORD, 0.0);

        // Set default rewards for placing blocks
        place_rewards.insert(Block::GRASS, 0.0);
        place_rewards.insert(Block::DIRT, 0.0);
        place_rewards.insert(Block::STONE, 0.0);
        place_rewards.insert(Block::COBBELSTONE, 0.0);
        place_rewards.insert(Block::OAKLOG, 0.0);
        place_rewards.insert(Block::OAKLEAVES, 0.0);
        place_rewards.insert(Block::SAND, 0.0);
        place_rewards.insert(Block::WATER, 0.0);
        place_rewards.insert(Block::SWORD, 0.0);

        Self {
            break_rewards,
            movement_reward: 0.0, // No reward for just moving
            place_rewards,
        }
    }
}

/// Manages reward calculation based on game events.
/// 
/// The `RewardManager` uses a `RewardConfig` to assign point values to different
/// game events. By default, breaking blocks gives rewards (stone: 5.0, dirt: 1.0, etc.),
/// while movement and placing blocks give 0 reward.
/// 
/// # Example
/// ```rust
/// use rusteze_core::reward_manager::RewardManager;
/// use rusteze_core::events::GameEvent;
/// use rusteze_core::world::block_kind::Block;
/// 
/// let manager = RewardManager::new();
/// let events = vec![
///     GameEvent::BlockBroken { block_type: Block::STONE },
/// ];
/// let reward = manager.calculate_reward(&events);
/// assert_eq!(reward, 5.0);
/// ```
pub struct RewardManager {
    config: RewardConfig,
}

impl RewardManager {
    /// Create a new reward manager with default configuration
    pub fn new() -> Self {
        Self {
            config: RewardConfig::default(),
        }
    }

    /// Create a reward manager with custom configuration
    pub fn with_config(config: RewardConfig) -> Self {
        Self { config }
    }

    /// Calculate total reward from a list of events
    pub fn calculate_reward(&self, events: &[GameEvent]) -> f32 {
        let mut total = 0.0;

        for event in events {
            match event {
                GameEvent::BlockBroken { block_type } => {
                    if let Some(reward) = self.config.break_rewards.get(block_type) {
                        total += reward;
                    }
                }
                GameEvent::PlayerMoved { distance } => {
                    total += distance * self.config.movement_reward;
                }
                GameEvent::BlockPlaced { block_type } => {
                    if let Some(reward) = self.config.place_rewards.get(block_type) {
                        total += reward;
                    }
                }
            }
        }

        total
    }
}
