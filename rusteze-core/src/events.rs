use crate::world::block_kind::Block;

/// Represents game events that can generate rewards
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// A block was broken
    BlockBroken { block_type: Block },

    /// Player moved a certain distance
    PlayerMoved { distance: f32 },

    /// Block was placed
    BlockPlaced { block_type: Block },
}
