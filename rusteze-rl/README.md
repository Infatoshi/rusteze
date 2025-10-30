# Rusteze RL Training Package

This package provides RL training tools and utilities for the Rusteze environment.

## Installation

```bash
# Install rusteze (from parent directory or via pip)
cd ../rusteze
uv pip install -e .

# Install RL dependencies
uv pip install stable-baselines3[extra] gymnasium opencv-python pynput

# Install this package
uv pip install -e .
```

## Training

Train a PPO agent:

```bash
python train.py
```

View training progress:

```bash
tensorboard --logdir logs
```

## Testing Trained Models

Test a trained model:

```bash
python test_trained_model.py --model_path ./models/rusteze_ppo_best_model --render
```

## Human Data Collection

Record human play data:

```bash
python record_human_play.py --duration 60 --output human_data.pkl
```

