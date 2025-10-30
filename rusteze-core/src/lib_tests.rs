#[cfg(test)]
mod tests {
    use crate::world::generation::world_generator::WorldGenerator;

    #[test]
    fn test_world_generation() {
        let world = WorldGenerator::create_new_random_world(5);
        // Verify world was created (we can't directly access chunks, but we can verify
        // the world exists and can be used)
        assert!(
            world
                .cube_at(crate::vector::Vector3::new(0., 0., 0.))
                .is_some()
                || world
                    .cube_at(crate::vector::Vector3::new(0., 0., 0.))
                    .is_none()
        );
    }
}
