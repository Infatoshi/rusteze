from setuptools import setup, find_packages

setup(
    name="rusteze-rl",
    version="0.1.0",
    description="RL training tools for Rusteze environment",
    packages=find_packages(),
    install_requires=[
        "rusteze",
        "stable-baselines3[extra]",
        "gymnasium",
        "opencv-python",
        "pynput",
        "numpy",
    ],
    python_requires=">=3.8",
)

