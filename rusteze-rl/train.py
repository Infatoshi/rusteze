#!/usr/bin/env python3
"""
Training script for Rusteze using stable-baselines3.

This script trains a PPO agent to learn to break log blocks in the Rusteze environment.
"""

import os
import time
from stable_baselines3 import PPO
from stable_baselines3.common.callbacks import CheckpointCallback, EvalCallback
from stable_baselines3.common.monitor import Monitor
from stable_baselines3.common.vec_env import DummyVecEnv
from rusteze_rl.wrapper import RustezeVecEnv

def make_env(seed: int = 0):
    """Create a single environment."""
    def _init():
        env = RustezeVecEnv(num_envs=1, seed=seed)
        return env
    return _init

def main():
    """Main training loop."""
    print("=" * 60)
    print("Rusteze RL Training")
    print("=" * 60)
    
    # Create log directory
    log_dir = "./logs"
    os.makedirs(log_dir, exist_ok=True)
    
    # Create models directory
    model_dir = "./models"
    os.makedirs(model_dir, exist_ok=True)
    
    # Create vectorized environment
    # Using 8 parallel environments for faster training
    print(f"\nCreating vectorized environment with 8 parallel environments...")
    print("Note: Reward configuration is set in Rust code.")
    print("  - Log blocks (OAKLOG): 3.0 reward")
    print("  - Stone blocks: 5.0 reward")
    print("  - Movement: 0.0 reward (no movement reward by default)")
    
    train_env = DummyVecEnv([make_env(seed=i) for i in range(8)])
    
    # Create evaluation environment
    eval_env = DummyVecEnv([make_env(seed=100)])
    
    # Wrap evaluation environment with Monitor
    eval_env = Monitor(eval_env, log_dir)
    
    print(f"\nObservation space: {train_env.observation_space}")
    print(f"Action space: {train_env.action_space}")
    
    # Create PPO model
    print("\nCreating PPO model...")
    model = PPO(
        "CnnPolicy",  # Use CNN policy for image observations
        train_env,
        verbose=1,
        tensorboard_log=log_dir,
        learning_rate=3e-4,
        n_steps=2048,
        batch_size=64,
        n_epochs=10,
        gamma=0.99,
        gae_lambda=0.95,
        clip_range=0.2,
        ent_coef=0.01,
        vf_coef=0.5,
        max_grad_norm=0.5,
    )
    
    # Create callbacks
    checkpoint_callback = CheckpointCallback(
        save_freq=10000,
        save_path=model_dir,
        name_prefix="rusteze_ppo",
    )
    
    eval_callback = EvalCallback(
        eval_env,
        best_model_save_path=model_dir,
        log_path=log_dir,
        eval_freq=5000,
        deterministic=True,
        render=False,
    )
    
    # Train the model
    print("\n" + "=" * 60)
    print("Starting training...")
    print("=" * 60)
    print(f"Total timesteps: 1,000,000")
    print(f"Checkpoints will be saved to: {model_dir}")
    print(f"Tensorboard logs: {log_dir}")
    print("\nPress Ctrl+C to stop training early.\n")
    
    start_time = time.time()
    
    try:
        model.learn(
            total_timesteps=1_000_000,
            callback=[checkpoint_callback, eval_callback],
            progress_bar=True,
        )
    except KeyboardInterrupt:
        print("\n\nTraining interrupted by user.")
    
    elapsed_time = time.time() - start_time
    
    # Save final model
    final_model_path = os.path.join(model_dir, "rusteze_ppo_final")
    print(f"\nSaving final model to {final_model_path}...")
    model.save(final_model_path)
    
    print("\n" + "=" * 60)
    print("Training completed!")
    print("=" * 60)
    print(f"Total training time: {elapsed_time:.2f} seconds")
    print(f"Final model saved to: {final_model_path}")
    print(f"\nTo view training progress:")
    print(f"  tensorboard --logdir {log_dir}")
    print(f"\nTo load and test the model:")
    print(f"  python test_trained_model.py")

if __name__ == "__main__":
    main()

