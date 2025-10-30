#!/usr/bin/env python3
"""Test script for parallel multi-environment execution."""

import rusteze
import numpy as np

def test_parallel():
    """Test MultiRustezeEnv with 4 parallel environments."""
    print("Creating MultiRustezeEnv with 4 environments...")
    multi_env = rusteze.MultiRustezeEnv(num_envs=4, seed=42)
    
    print(f"Number of environments: {multi_env.num_envs()}")
    
    print("Resetting all environments...")
    observations = multi_env.reset_all()
    print(f"Got {len(observations)} observations")
    for i, obs in enumerate(observations):
        print(f"  Env {i}: shape={obs.shape}, dtype={obs.dtype}")
    
    print("\nStepping all environments...")
    # Create actions for each environment
    actions = [
        {"forward": True, "camera": [0.0, 0.0], "attack": False},   # Env 0: move forward
        {"forward": False, "camera": [5.0, 0.0], "attack": False},  # Env 1: turn right
        {"forward": False, "camera": [-5.0, 0.0], "attack": False}, # Env 2: turn left
        {"forward": False, "camera": [0.0, 0.0], "attack": True},   # Env 3: attack
    ]
    
    obs_batch, rewards, dones = multi_env.step_all(actions)
    
    print(f"Got {len(obs_batch)} observations")
    print(f"Got {len(rewards)} rewards: {rewards}")
    print(f"Got {len(dones)} done flags: {dones}")
    
    # Verify all observations have correct shape
    for i, obs in enumerate(obs_batch):
        assert obs.shape == (360, 640, 3), f"Env {i}: expected shape (360, 640, 3), got {obs.shape}"
        assert obs.dtype == np.uint8, f"Env {i}: expected dtype uint8, got {obs.dtype}"
    
    print("\n✅ All tests passed!")

if __name__ == "__main__":
    test_parallel()

