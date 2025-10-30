# Rusteze

A headless Minecraft-like game engine written in Rust with Python bindings.

## Installation

### Prerequisites

- Rust toolchain (install from https://rustup.rs/)
- Python 3.11+
- `uv` (Python package manager)

### Install from source

```bash
git clone <repository-url>
cd rusteze
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
uv pip install .
```

## Basic Usage

```python
import rusteze
import numpy as np
import cv2  # opencv-python

# Create environment
env = rusteze.Env(seed=123)

# Reset and get initial observation
obs = env.reset()
print(f"Observation shape: {obs.shape}")  # Should be (360, 640, 3)

# Save first frame
cv2.imwrite("initial.png", obs)

# Run a simple interactive loop
for i in range(100):
    # Create action dictionary for player movement
    action = {
        "camera": [5.0, 0.0],  # Turn right 5 degrees
        "forward": True,       # Move forward
        "jump": False,
        "attack": False,
    }
    
    obs, reward, done = env.step(action)
    
    # Display the observation
    cv2.imshow("Rusteze View", obs)
    if cv2.waitKey(1) & 0xFF == ord('q'):
        break
    
    if done:
        break

cv2.imwrite("final.png", obs)
cv2.destroyAllWindows()
```

## Movement and Interaction Example

```python
import rusteze
import cv2

env = rusteze.Env(seed=42)
obs = env.reset()
cv2.imwrite("movement_before.png", obs)

# Turn right strongly
action = {
    "camera": [15.0, 0.0],  # Turn right 15 degrees
    "forward": False,
    "back": False,
    "left": False,
    "right": False,
    "jump": False,
    "attack": False,
}
obs, _, _ = env.step(action)
cv2.imwrite("movement_after.png", obs)

# Move forward for a few steps
for _ in range(10):
    action = {
        "camera": [0.0, 0.0],
        "forward": True,
        "attack": False,
    }
    obs, _, _ = env.step(action)

# Break a block (look at one and attack)
action = {
    "camera": [0.0, 0.0],
    "forward": False,
    "attack": True,  # Break the block the player is looking at
}
for _ in range(5):  # Need to hold attack for a few frames
    obs, _, _ = env.step(action)

cv2.imwrite("interaction_after.png", obs)
cv2.destroyAllWindows()
```

## Action Format

Actions can be passed as:

1. **Python dictionary** (recommended):
```python
action = {
    "camera": [horizontal, vertical],  # Optional: Camera rotation in degrees
    "forward": bool,                   # Move forward
    "back": bool,                      # Move backward
    "left": bool,                      # Strafe left
    "right": bool,                     # Strafe right
    "jump": bool,                      # Jump
    "attack": bool,                    # Break block the player is looking at
}
env.step(action)
```

2. **JSON string**:
```python
# PlayerInput action
env.step('{"PlayerInput": {"input": {"camera": [5.0, 0.0], "forward": true, "attack": false}}}')

# Direct world modification actions
env.step('{"Destroy": {"at": [0.0, 1.0, 0.0]}}')
env.step('{"Add": {"at": [0.0, 2.0, 0.0], "block": "GRASS"}}')
```

3. **None** (Noop):
```python
env.step(None)  # No action, just advance simulation
```

## API Reference

### `RustezeEnv`

- `reset()` -> `np.ndarray`: Reset the environment and return the initial observation (360, 640, 3) RGB image.
- `step(action)` -> `(np.ndarray, float, bool)`: Step the environment with an action. Returns (observation, reward, done).
- `width() -> int`: Returns the observation width (640).
- `height() -> int`: Returns the observation height (360).

## Development

To build and test the Rust core:

```bash
cd rusteze-core
cargo test
```

## License

[Add your license here]


