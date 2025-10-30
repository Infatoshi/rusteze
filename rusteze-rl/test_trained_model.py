#!/usr/bin/env python3
"""
Test script for trained Rusteze model.
Loads a trained model and runs it in the environment, displaying the results.
"""

import argparse
import numpy as np
import cv2
from stable_baselines3 import PPO
from rusteze_rl.wrapper import RustezeVecEnv

def main():
    parser = argparse.ArgumentParser(description="Test a trained Rusteze model")
    parser.add_argument(
        "--model_path",
        type=str,
        default="./models/rusteze_ppo_best_model",
        help="Path to the trained model",
    )
    parser.add_argument(
        "--episodes",
        type=int,
        default=3,
        help="Number of episodes to run",
    )
    parser.add_argument(
        "--render",
        action="store_true",
        help="Display the agent's view",
    )
    
    args = parser.parse_args()
    
    print(f"Loading model from {args.model_path}...")
    try:
        model = PPO.load(args.model_path)
    except FileNotFoundError:
        print(f"Error: Model not found at {args.model_path}")
        print("Available models:")
        import os
        if os.path.exists("./models"):
            for f in os.listdir("./models"):
                if f.endswith(".zip"):
                    print(f"  ./models/{f}")
        return
    
    # Create environment
    env = RustezeVecEnv(num_envs=1, seed=42)
    
    total_rewards = []
    
    for episode in range(args.episodes):
        print(f"\nEpisode {episode + 1}/{args.episodes}")
        obs, info = env.reset()
        done = False
        episode_reward = 0.0
        step_count = 0
        
        while not done:
            # Get action from model
            action, _ = model.predict(obs, deterministic=True)
            
            # Step environment
            obs, reward, terminated, truncated, info = env.step(action)
            done = terminated[0] or truncated[0]
            
            episode_reward += reward[0]
            step_count += 1
            
            # Render if requested
            if args.render:
                frame = obs[0]  # Get first (and only) environment's observation
                frame_bgr = cv2.cvtColor(frame, cv2.COLOR_RGB2BGR)
                cv2.imshow("Rusteze Agent", frame_bgr)
                if cv2.waitKey(1) & 0xFF == ord('q'):
                    print("\nQuitting...")
                    return
            
            if step_count % 100 == 0:
                print(f"  Step {step_count}, Reward: {episode_reward:.2f}")
        
        total_rewards.append(episode_reward)
        print(f"Episode {episode + 1} completed:")
        print(f"  Total steps: {step_count}")
        print(f"  Total reward: {episode_reward:.2f}")
    
    if args.render:
        cv2.destroyAllWindows()
    
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    print(f"Episodes: {args.episodes}")
    print(f"Mean reward: {np.mean(total_rewards):.2f}")
    print(f"Std reward: {np.std(total_rewards):.2f}")
    print(f"Min reward: {np.min(total_rewards):.2f}")
    print(f"Max reward: {np.max(total_rewards):.2f}")

if __name__ == "__main__":
    main()

