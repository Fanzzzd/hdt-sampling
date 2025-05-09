# HDT Sampling (Hierarchical Dart Throwing)

[![CI](https://github.com/Fanzzzd/hdt_sampling/actions/workflows/CI.yml/badge.svg)](https://github.com/Fanzzzd/hdt_sampling/actions/workflows/CI.yml)
[![PyPI](https://img.shields.io/pypi/v/hdt-sampling?logo=pypi)](https://pypi.org/project/hdt-sampling/)

[中文说明 (Chinese Readme)](README_zh.md)

A fast Python library implemented in Rust for generating 2D Poisson Disk point sets using the Hierarchical Dart Throwing (HDT) algorithm. Poisson Disk sampling produces points that are tightly packed but no two points are closer than a specified minimum distance, resulting in a high-quality blue noise distribution.

This implementation is based on the paper:

> K. B. White, D. Cline and P. K. Egbert, "Poisson Disk Point Sets by Hierarchical Dart Throwing," *2007 IEEE Symposium on Interactive Ray Tracing*, Ulm, Germany, 2007, pp. 129-132, doi: 10.1109/RT.2007.4342600.
> ([Link to paper on BYU ScholarsArchive](https://scholarsarchive.byu.edu/facpub/237))

## Features

*   Generates 2D Poisson Disk samples within a rectangular domain.
*   Fast implementation in Rust, callable from Python.
*   Produces maximal point sets (fills the space as much as possible according to the minimum distance rule).
*   Simple API.

## Performance

On an Apple M3 Max chip, generating approximately 170,000 points in a 10000x10000 domain with a minimum distance of 20.0 takes about **0.78 seconds**.

## Installation

You can install the package from PyPI:

```bash
pip install hdt-sampling
```

Alternatively, if you have Rust and Maturin installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/Fanzzzd/hdt_sampling.git
cd hdt_sampling

# Install maturin (if you don't have it)
# pip install maturin

# Build and install in your current Python environment
maturin develop --release
# Or build a wheel for distribution
# maturin build --release
```

## Usage

```python
import time
import matplotlib.pyplot as plt
from hdt_sampling import HDTSampler

# --- Parameters ---
width = 1000.0  # Domain width
height = 1000.0 # Domain height
min_dist = 15.0 # Minimum distance between points

# --- Generate Points ---
print("Initializing HDT sampler...")
start_init = time.time()
sampler = HDTSampler(width, height, min_dist)
end_init = time.time()
print(f"Initialization took: {end_init - start_init:.4f} seconds")

print("Generating points...")
start_gen = time.time()
points = sampler.generate() # Returns a list of (x, y) tuples
end_gen = time.time()
print(f"Generation took: {end_gen - start_gen:.4f} seconds")
print(f"Generated {len(points)} points.")

# --- Plotting ---
if points:
    x_coords, y_coords = zip(*points)

    plt.figure(figsize=(8, 8))
    plt.scatter(x_coords, y_coords, s=5, c='blue', alpha=0.7)
    plt.xlim(0, width)
    plt.ylim(0, height)
    plt.xlabel("X")
    plt.ylabel("Y")
    plt.title(f"Poisson Disk Points (r={min_dist}) - {len(points)} points")
    plt.gca().set_aspect('equal', adjustable='box')
    plt.grid(True, linestyle='--', alpha=0.4)
    plt.show()
else:
    print("No points were generated.")
```

See the `example/example.ipynb` file for a more detailed example including Fourier analysis plots.

## License

This project is licensed under the MIT License. See the LICENSE file for details.