from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="rusteze",
    version="0.1.0",
    rust_extensions=[RustExtension("rusteze.rusteze_core", "rusteze-core/Cargo.toml", binding=Binding.PyO3, features=["extension-module"])],
    packages=["rusteze"],
    zip_safe=False,
)

