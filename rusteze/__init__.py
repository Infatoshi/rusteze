# Rusteze Python package
from .rusteze_core import RustezeEnv, MultiRustezeEnv

__all__ = ["RustezeEnv", "MultiRustezeEnv"]

# Create a convenience alias
Env = RustezeEnv
MultiEnv = MultiRustezeEnv


