from setuptools import setup
from os.path import join
import os

project_name = "xiron_py"

setup(
    name=project_name,
    version="0.1.0",
    url="https://github.com/SuhrudhSarathy/xiron",
    author="Suhrudh Sarathy",
    author_email="suhrudhsarathy@gmail.com",
    description="Python Bindings for xiron",
    license="MIT",
    install_requires=["grpcio", "grpcio-tools"],
    packages=[
        project_name,
    ],
    zip_safe=True
)