import time
import numpy as np
import matplotlib.pyplot as plt
from hdt_sampling import HDTSampler # Import the renamed class from the renamed package

# --- Parameters ---
width = 10000.0
height = 10000.0
min_dist = 20.0
plot_limit = 1000.0 # Only plot points within this boundary

# --- Generate Points using Rust HDT ---
print("Initializing HDT sampler...") # Updated print message
start_init = time.time()
hdt_sampler = HDTSampler(width, height, min_dist) # Use the renamed class
end_init = time.time()
print(f"Initialization took: {end_init - start_init:.4f} seconds")

print("Generating points with HDT sampler...") # Updated print message
start_gen = time.time()
valid_points = hdt_sampler.generate() # Returns Vec<(f64, f64)>
end_gen = time.time()
print(f"Generation took: {end_gen - start_gen:.4f} seconds")
print(f"Generated {len(valid_points)} points.")


# --- Plotting (only a small window) ---
if valid_points:
    # Filter points within the plotting window
    points_to_plot = [(x, y) for x, y in valid_points if 0 <= x <= plot_limit and 0 <= y <= plot_limit]
    print(f"Found {len(points_to_plot)} points within the {plot_limit}x{plot_limit} area for plotting.")

    if points_to_plot:
        x_coords, y_coords = zip(*points_to_plot)

        # Create a figure with two subplots - for spatial and spectral analysis
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 8))

        # Spatial domain plot
        ax1.scatter(x_coords, y_coords, s=5, c='red', alpha=0.7)
        ax1.set_xlim(0, plot_limit)
        ax1.set_ylim(0, plot_limit)
        ax1.set_xlabel("X")
        ax1.set_ylabel("Y")
        ax1.set_title(f"Poisson Disk Points (r={min_dist})")
        ax1.set_aspect('equal', adjustable='box')
        ax1.grid(True, linestyle='--', alpha=0.4)

        # Fourier analysis
        # Create a grid representation of points
        grid_size = 512
        grid = np.zeros((grid_size, grid_size))

        # Scale coordinates to grid
        scale_x = grid_size / plot_limit
        scale_y = grid_size / plot_limit

        for x, y in points_to_plot:
            gx = int(x * scale_x)
            gy = int(y * scale_y)
            if 0 <= gx < grid_size and 0 <= gy < grid_size:
                grid[gy, gx] = 1  # Note: y is row, x is column

        # Compute 2D FFT and shift the zero-frequency component to center
        fft_result = np.fft.fft2(grid)
        fft_shifted = np.fft.fftshift(fft_result)

        # Compute the magnitude spectrum (log scale for better visualization)
        magnitude_spectrum = np.log(np.abs(fft_shifted) + 1)

        # Plot the frequency domain
        im = ax2.imshow(magnitude_spectrum, cmap='viridis')
        ax2.set_title('Fourier Transform (Magnitude Spectrum)')
        ax2.set_xlabel('Frequency')
        ax2.set_ylabel('Frequency')
        fig.colorbar(im, ax=ax2, label='Log Magnitude')

        # Add radially averaged power spectrum
        center_y, center_x = grid_size // 2, grid_size // 2
        y_coords_grid, x_coords_grid = np.ogrid[:grid_size, :grid_size]
        r = np.sqrt((x_coords_grid - center_x)**2 + (y_coords_grid - center_y)**2)
        r = r.astype(int)

        # Calculate the radial profile
        tbin = np.bincount(r.ravel(), magnitude_spectrum.ravel())
        nr = np.bincount(r.ravel())
        # Avoid division by zero for bins with no samples
        radial_profile = np.zeros_like(tbin, dtype=float)
        non_zero_nr = nr > 0
        radial_profile[non_zero_nr] = tbin[non_zero_nr] / nr[non_zero_nr]


        # Add an inset for the radial profile
        ax_inset = fig.add_axes([0.76, 0.15, 0.18, 0.25])
        ax_inset.plot(radial_profile[:grid_size//2])
        ax_inset.set_title('Radial Power Spectrum')
        ax_inset.set_xlabel('Frequency')
        ax_inset.set_ylabel('Power')

        plt.tight_layout(rect=[0, 0, 0.75, 1]) # Adjust layout to prevent overlap with inset
        plt.show()
    else:
        print(f"No points found within the {plot_limit}x{plot_limit} area to plot.")
else:
    print("No points were generated.")