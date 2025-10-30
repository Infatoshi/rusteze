#!/usr/bin/env python3
"""
Human-in-the-loop data collection script for Rusteze.

This script allows a human player to control the agent and records
(observation, action, reward, done) tuples for imitation learning.
"""

import argparse
import numpy as np
import cv2
import time
from collections import deque
from pynput import keyboard, mouse
import rusteze
import pickle
import os

class HumanPlayer:
    """Handles human input for Rusteze environment."""
    
    def __init__(self):
        self.keys_pressed = set()
        self.mouse_delta = [0.0, 0.0]
        self.mouse_last_pos = None
        
        # Setup keyboard listener
        self.keyboard_listener = keyboard.Listener(
            on_press=self._on_key_press,
            on_release=self._on_key_release,
        )
        self.keyboard_listener.start()
        
        # Setup mouse listener
        self.mouse_listener = mouse.Listener(
            on_move=self._on_mouse_move,
        )
        self.mouse_listener.start()
    
    def _on_key_press(self, key):
        """Handle key press."""
        try:
            if hasattr(key, 'char') and key.char:
                self.keys_pressed.add(key.char.lower())
            elif key == keyboard.Key.space:
                self.keys_pressed.add('space')
            elif key == keyboard.Key.shift:
                self.keys_pressed.add('shift')
        except AttributeError:
            pass
    
    def _on_key_release(self, key):
        """Handle key release."""
        try:
            if hasattr(key, 'char') and key.char:
                self.keys_pressed.discard(key.char.lower())
            elif key == keyboard.Key.space:
                self.keys_pressed.discard('space')
            elif key == keyboard.Key.shift:
                self.keys_pressed.discard('shift')
        except AttributeError:
            pass
    
    def _on_mouse_move(self, x, y):
        """Handle mouse movement."""
        if self.mouse_last_pos is None:
            self.mouse_last_pos = (x, y)
            return
        
        dx = x - self.mouse_last_pos[0]
        dy = y - self.mouse_last_pos[1]
        
        # Convert pixel movement to camera rotation
        # Scale factor: 0.1 degrees per pixel
        self.mouse_delta[0] += dx * 0.1
        self.mouse_delta[1] += dy * 0.1
        
        self.mouse_last_pos = (x, y)
    
    def get_action(self) -> dict:
        """Convert current input state to rusteze action."""
        action = {
            "forward": 'w' in self.keys_pressed,
            "back": 's' in self.keys_pressed,
            "left": 'a' in self.keys_pressed,
            "right": 'd' in self.keys_pressed,
            "jump": 'space' in self.keys_pressed,
            "attack": 'shift' in self.keys_pressed or mouse.Button.left in self.keys_pressed,
            "camera": [self.mouse_delta[0], self.mouse_delta[1]],
        }
        
        # Reset mouse delta after reading
        self.mouse_delta = [0.0, 0.0]
        
        return action
    
    def stop(self):
        """Stop input listeners."""
        self.keyboard_listener.stop()
        self.mouse_listener.stop()

def main():
    parser = argparse.ArgumentParser(description="Record human play data")
    parser.add_argument(
        "--output",
        type=str,
        default="./human_play_data.pkl",
        help="Output file path",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed",
    )
    parser.add_argument(
        "--duration",
        type=int,
        default=60,
        help="Recording duration in seconds",
    )
    
    args = parser.parse_args()
    
    print("=" * 60)
    print("Rusteze Human Play Recording")
    print("=" * 60)
    print("\nControls:")
    print("  W/A/S/D - Move")
    print("  Space - Jump")
    print("  Shift or Left Click - Attack/Break")
    print("  Mouse - Look around")
    print("  Q - Quit recording")
    print(f"\nRecording for {args.duration} seconds...")
    print("Recording will start in 3 seconds...")
    print("(Switch to the game window when it appears)\n")
    
    time.sleep(3)
    
    # Initialize environment
    env = rusteze.Env(seed=args.seed)
    obs = env.reset()
    
    # Initialize human player
    player = HumanPlayer()
    
    # Data storage
    data = {
        "observations": [],
        "actions": [],
        "rewards": [],
        "dones": [],
    }
    
    # Recording parameters
    start_time = time.time()
    frame_count = 0
    quit_requested = False
    
    print("Recording started! Press Q to quit early.\n")
    
    try:
        while True:
            current_time = time.time()
            elapsed = current_time - start_time
            
            if elapsed >= args.duration:
                print(f"\nRecording duration ({args.duration}s) reached.")
                break
            
            # Check for quit
            if 'q' in player.keys_pressed:
                print("\nQuit requested by user.")
                quit_requested = True
                break
            
            # Get action from human player
            action = player.get_action()
            
            # Step environment
            next_obs, reward, done = env.step(action)
            
            # Store data
            data["observations"].append(obs.copy())
            data["actions"].append(action.copy())
            data["rewards"].append(reward)
            data["dones"].append(done)
            
            # Update observation
            obs = next_obs
            
            # Display observation
            frame_bgr = cv2.cvtColor(obs, cv2.COLOR_RGB2BGR)
            cv2.imshow("Rusteze - Recording", frame_bgr)
            
            # Handle done
            if done:
                obs = env.reset()
            
            frame_count += 1
            
            # Print progress every 100 frames
            if frame_count % 100 == 0:
                print(f"  Frames: {frame_count}, Time: {elapsed:.1f}s / {args.duration}s")
            
            # Small delay to maintain frame rate
            if cv2.waitKey(1) & 0xFF == ord('q'):
                quit_requested = True
                break
            
            time.sleep(1.0 / 60.0)  # ~60 FPS
    
    except KeyboardInterrupt:
        print("\n\nRecording interrupted by user.")
    
    finally:
        # Cleanup
        player.stop()
        cv2.destroyAllWindows()
        
        # Save data
        print(f"\nSaving data to {args.output}...")
        with open(args.output, 'wb') as f:
            pickle.dump(data, f)
        
        print("\n" + "=" * 60)
        print("Recording Summary")
        print("=" * 60)
        print(f"Total frames: {frame_count}")
        print(f"Total time: {time.time() - start_time:.2f} seconds")
        print(f"Data saved to: {args.output}")
        print(f"\nData statistics:")
        print(f"  Observations: {len(data['observations'])}")
        print(f"  Actions: {len(data['actions'])}")
        print(f"  Rewards: {len(data['rewards'])}")
        print(f"  Mean reward: {np.mean(data['rewards']):.2f}")
        print(f"  Total reward: {np.sum(data['rewards']):.2f}")

if __name__ == "__main__":
    main()

