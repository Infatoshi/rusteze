#!/usr/bin/env python3
"""
Gymnasium-compatible wrapper for MultiRustezeEnv.
This allows rusteze to work with standard RL libraries like stable-baselines3.
"""

import gymnasium as gym
from gymnasium import spaces
import numpy as np
from typing import Tuple, Optional, Dict, Any
import rusteze


class RustezeVecEnv(gym.vector.VectorEnv):
    """
    A gymnasium.vector.VectorEnv wrapper for MultiRustezeEnv.
    
    This wrapper enables rusteze to work with stable-baselines3 and other
    RL libraries that expect the gymnasium vectorized environment interface.
    """
    
    metadata = {"render_modes": ["rgb_array"], "render_fps": 60}
    
    def __init__(
        self,
        num_envs: int = 8,
        seed: Optional[int] = None,
        break_log_reward: float = 10.0,
        move_reward_per_meter: float = 0.01,
    ):
        """
        Initialize the vectorized Rusteze environment.
        
        Args:
            num_envs: Number of parallel environments
            seed: Random seed for world generation
            break_log_reward: Reward for breaking a log block
            move_reward_per_meter: Reward per meter moved
        """
        self.num_envs = num_envs
        self.seed = seed or 42
        
        # Initialize the multi-environment
        self.multi_env = rusteze.MultiRustezeEnv(num_envs=num_envs, seed=self.seed)
        
        # Observation space: RGB images (height, width, channels)
        self.observation_space = spaces.Box(
            low=0,
            high=255,
            shape=(num_envs, 360, 640, 3),
            dtype=np.uint8,
        )
        
        # Action space: Dictionary with player input fields
        # We'll use a flattened discrete action space for simplicity
        # Actions: [noop, forward, back, left, right, forward+left, forward+right, attack]
        self.action_space = spaces.Discrete(8)
        
        # Store reward configuration
        self.break_log_reward = break_log_reward
        self.move_reward_per_meter = move_reward_per_meter
        
        # Store current observations
        self._observations = None
        
    def reset(
        self,
        seed: Optional[int] = None,
        options: Optional[Dict[str, Any]] = None,
    ) -> Tuple[np.ndarray, Dict[str, Any]]:
        """
        Reset all environments.
        
        Returns:
            Tuple of (observations, infos)
        """
        if seed is not None:
            self.seed = seed
        
        observations = self.multi_env.reset_all()
        
        # Convert to numpy array
        obs_array = np.stack([np.array(obs) for obs in observations], axis=0)
        self._observations = obs_array
        
        infos = [{} for _ in range(self.num_envs)]
        
        return obs_array, infos
    
    def step(
        self, actions: np.ndarray
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, Dict[str, Any]]:
        """
        Step all environments.
        
        Args:
            actions: Array of discrete actions (one per environment)
            
        Returns:
            Tuple of (observations, rewards, terminateds, truncateds, infos)
        """
        # Convert discrete actions to action dictionaries
        action_dicts = []
        for action in actions:
            action_dict = self._discrete_to_action_dict(action)
            action_dicts.append(action_dict)
        
        # Step all environments
        obs_list, rewards, dones = self.multi_env.step_all(action_dicts)
        
        # Convert observations to numpy array
        obs_array = np.stack([np.array(obs) for obs in obs_list], axis=0)
        self._observations = obs_array
        
        # Convert rewards to numpy array
        rewards_array = np.array(rewards, dtype=np.float32)
        
        # Split dones into terminated and truncated
        terminateds = np.array(dones, dtype=bool)
        truncateds = np.zeros_like(terminateds, dtype=bool)
        
        infos = [{} for _ in range(self.num_envs)]
        
        return obs_array, rewards_array, terminateds, truncateds, infos
    
    def _discrete_to_action_dict(self, action: int) -> Dict[str, Any]:
        """
        Convert a discrete action integer to a rusteze action dictionary.
        
        Actions:
        0: Noop
        1: Forward
        2: Back
        3: Left
        4: Right
        5: Forward + Left
        6: Forward + Right
        7: Attack
        """
        action_dict = {
            "forward": False,
            "back": False,
            "left": False,
            "right": False,
            "jump": False,
            "attack": False,
            "camera": [0.0, 0.0],
        }
        
        if action == 0:  # Noop
            pass
        elif action == 1:  # Forward
            action_dict["forward"] = True
        elif action == 2:  # Back
            action_dict["back"] = True
        elif action == 3:  # Left
            action_dict["left"] = True
        elif action == 4:  # Right
            action_dict["right"] = True
        elif action == 5:  # Forward + Left
            action_dict["forward"] = True
            action_dict["left"] = True
        elif action == 6:  # Forward + Right
            action_dict["forward"] = True
            action_dict["right"] = True
        elif action == 7:  # Attack
            action_dict["attack"] = True
        
        return action_dict
    
    def render(self) -> Optional[np.ndarray]:
        """
        Render the environment.
        
        Returns:
            RGB array of the first environment's observation
        """
        if self._observations is None:
            return None
        return self._observations[0]
    
    def close(self):
        """Clean up resources."""
        pass

