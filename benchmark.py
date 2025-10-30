#!/usr/bin/env python3
"""Benchmark script for Rusteze environment performance."""

import time
import rusteze

def benchmark(num_steps=10000):
    """Run benchmark and return steps per second."""
    print(f"Initializing environment...")
    env = rusteze.Env(seed=42)
    
    print(f"Resetting environment...")
    env.reset()
    
    # Simple constant action (move forward)
    action = {
        "forward": True,
        "camera": [0.0, 0.0],
        "attack": False,
    }
    
    print(f"Running {num_steps} steps...")
    start_time = time.time()
    
    for i in range(num_steps):
        obs, reward, done = env.step(action)
        if done:
            env.reset()
        
        # Print progress every 1000 steps
        if (i + 1) % 1000 == 0:
            elapsed = time.time() - start_time
            sps = (i + 1) / elapsed
            print(f"  Step {i + 1}/{num_steps} - {sps:.1f} SPS")
    
    end_time = time.time()
    total_time = end_time - start_time
    sps = num_steps / total_time
    
    print(f"\nBenchmark complete!")
    print(f"Total time: {total_time:.2f} seconds")
    print(f"Steps per second: {sps:.2f} SPS")
    
    return sps

if __name__ == "__main__":
    benchmark()

