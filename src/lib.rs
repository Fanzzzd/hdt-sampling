use pyo3::prelude::*;
use rand::prelude::*; 
use rand::rngs::StdRng;
use std::cmp::{max, min};

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug)]
struct Square {
    level: usize,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug)]
struct GridCell {
    indices: Vec<usize>, 
}

#[pyclass(name = "HDTSampler")]
struct HDTSampler { 
    width: f64,
    height: f64,
    r_squared: f64,
    cell_size: f64,
    grid_cols: usize,
    grid_rows: usize,
    grid: Vec<GridCell>,
    points: Vec<Point>,
    active_lists: Vec<Vec<Square>>,
    total_active_area: f64,
    b0: f64, 
    rng: StdRng,
}

impl HDTSampler {
    fn get_grid_coords(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_x = (x / self.cell_size).floor() as usize;
        let grid_y = (y / self.cell_size).floor() as usize;
        // Clamp coordinates to be within grid bounds
        (
            grid_x.min(self.grid_cols - 1),
            grid_y.min(self.grid_rows - 1),
        )
    }

    fn get_square_size(&self, level: usize) -> f64 {
        self.b0 / 2.0f64.powi(level as i32)
    }

    fn is_dart_valid(&self, x: f64, y: f64) -> bool {
        let (grid_x, grid_y) = self.get_grid_coords(x, y);

        let x_start = max(0, grid_x as i32 - 2) as usize;
        let x_end = min(self.grid_cols - 1, grid_x + 2);
        let y_start = max(0, grid_y as i32 - 2) as usize;
        let y_end = min(self.grid_rows - 1, grid_y + 2);

        for i in x_start..=x_end {
            for j in y_start..=y_end {
                let cell_idx = i + j * self.grid_cols;
                // Use checked access to grid
                if let Some(cell) = self.grid.get(cell_idx) {
                    for &pt_idx in &cell.indices {
                        // Use checked access to points
                        if let Some(existing_point) = self.points.get(pt_idx) {
                            let dx = x - existing_point.x;
                            let dy = y - existing_point.y;
                            let dist_sq = dx * dx + dy * dy;
                            if dist_sq < self.r_squared {
                                return false; // Too close
                            }
                        }
                    }
                }
            }
        }
        true 
    }

    fn get_farthest_corner_dist_sq(&self, square: &Square, px: f64, py: f64) -> f64 {
        let sq_size = self.get_square_size(square.level);
        let center_x = square.x + sq_size / 2.0;
        let center_y = square.y + sq_size / 2.0;

        let dx = (center_x - px).abs() + sq_size / 2.0;
        let dy = (center_y - py).abs() + sq_size / 2.0;
        dx * dx + dy * dy
    }

    fn is_square_covered(&self, square: &Square) -> bool {
        let sq_size = self.get_square_size(square.level);
        let center_x = square.x + sq_size / 2.0;
        let center_y = square.y + sq_size / 2.0;

        let (center_grid_x, center_grid_y) = self.get_grid_coords(center_x, center_y);

        let x_start = max(0, center_grid_x as i32 - 2) as usize;
        let x_end = min(self.grid_cols - 1, center_grid_x + 2);
        let y_start = max(0, center_grid_y as i32 - 2) as usize;
        let y_end = min(self.grid_rows - 1, center_grid_y + 2);

        for i in x_start..=x_end {
            for j in y_start..=y_end {
                let cell_idx = i + j * self.grid_cols;
                 // Use checked access to grid
                if let Some(cell) = self.grid.get(cell_idx) {
                    for &pt_idx in &cell.indices {
                        // Use checked access to points
                        if let Some(existing_point) = self.points.get(pt_idx) {
                           if self.get_farthest_corner_dist_sq(square, existing_point.x, existing_point.y) < self.r_squared {
                                return true; // Covered
                           }
                        }
                    }
                }
            }
        }
        false // Not covered
    }


    fn add_point(&mut self, x: f64, y: f64) {
        let pt_idx = self.points.len();
        self.points.push(Point { x, y });

        let (grid_x, grid_y) = self.get_grid_coords(x, y);
        let cell_idx = grid_x + grid_y * self.grid_cols;
        // Use checked access to grid
        if let Some(cell) = self.grid.get_mut(cell_idx) {
             cell.indices.push(pt_idx);
        } else {
            // This case should ideally not happen if grid is initialized correctly,
            // but handle it defensively.
            eprintln!("Warning: Attempted to access invalid grid cell index {}", cell_idx);
        }
    }

     fn ensure_active_list_level(&mut self, level: usize) {
        while self.active_lists.len() <= level {
            self.active_lists.push(Vec::new());
        }
    }

    fn add_child_square(&mut self, level: usize, x: f64, y: f64) {
        if x >= self.width || y >= self.height {
            return;
        }

        let square = Square { level, x, y };

        if self.is_square_covered(&square) {
            return;
        }

        self.ensure_active_list_level(level);

        // Use checked access to active_lists
        if let Some(list) = self.active_lists.get_mut(level) {
             list.push(square);
             let sq_size = self.get_square_size(level);
             self.total_active_area += sq_size * sq_size;
        } else {
             // Should not happen if ensure_active_list_level works correctly.
            eprintln!("Warning: Failed to get mutable access to active list for level {}", level);
        }
    }


    fn choose_active_square(&mut self) -> Option<(Square, usize)> {
        if self.total_active_area <= 1e-9 {
            return None;
        }

        // Use random_range instead of the gen_range(deprecated)
        let target_area = self.rng.random_range(0.0..self.total_active_area);
        let mut current_area_sum = 0.0;

        // Pre-calculate square sizes for each level to avoid immutable borrow later
        let square_sizes: Vec<f64> = (0..self.active_lists.len())
            .map(|level| self.get_square_size(level))
            .collect();

        for level_idx in 0..self.active_lists.len() {
             // Use checked access and handle potential empty list
             if let Some(squares_at_level) = self.active_lists.get_mut(level_idx) {
                 if squares_at_level.is_empty() {
                     continue;
                 }

                 let sq_size = square_sizes[level_idx];
                 let area_per_square = sq_size * sq_size;
                 let num_squares_at_level = squares_at_level.len();
                 let level_total_area = num_squares_at_level as f64 * area_per_square;

                 if current_area_sum + level_total_area > target_area {
                     let remaining_area = target_area - current_area_sum;
                     // Ensure division by non-zero area
                     let target_idx_in_level_f = if area_per_square > 0.0 {
                         remaining_area / area_per_square
                     } else {
                         0.0 // Avoid division by zero if area is somehow zero
                     };

                     let mut target_idx_in_level = target_idx_in_level_f.floor() as usize;

                     // Ensure index is within bounds due to potential floating point inaccuracies
                     target_idx_in_level = target_idx_in_level.min(num_squares_at_level - 1);


                     // Efficiently remove: swap with the last element and pop
                     let chosen_square = squares_at_level.swap_remove(target_idx_in_level);

                     self.total_active_area -= area_per_square;
                     return Some((chosen_square, level_idx));
                 }
                 current_area_sum += level_total_area;
             }
        }

        // Should not happen if total_active_area > 0
        eprintln!("Warning: Could not choose active square despite positive total area.");
        None
    }
}

#[pymethods]
impl HDTSampler { 
    #[new]
    fn new(width: f64, height: f64, r: f64) -> PyResult<Self> {
        if r <= 0.0 {
             return Err(pyo3::exceptions::PyValueError::new_err("r must be positive"));
        }
        let r_squared = r * r;
        let cell_size = r / std::f64::consts::SQRT_2;
        let grid_cols = ((width / cell_size).ceil() as usize).max(1);
        let grid_rows = ((height / cell_size).ceil() as usize).max(1);
        let grid = vec![GridCell { indices: Vec::new() }; grid_cols * grid_rows];

        let points = Vec::new();
        let mut active_lists: Vec<Vec<Square>> = vec![Vec::new()]; 
        let b0 = cell_size;
        let mut total_active_area = 0.0;

        let rng = StdRng::from_os_rng();

        // Initialize level 0 active squares
        let initial_capacity = grid_cols * grid_rows;
        if let Some(level0_list) = active_lists.get_mut(0) {
             level0_list.reserve(initial_capacity);
             for i in 0..grid_cols {
                 for j in 0..grid_rows {
                     let x = i as f64 * b0;
                     let y = j as f64 * b0;
                     if x < width && y < height {
                         level0_list.push(Square { level: 0, x, y });
                         total_active_area += b0 * b0;
                     }
                 }
             }
             level0_list.shrink_to_fit(); 
        }


        Ok(HDTSampler { 
            width,
            height,
            r_squared,
            cell_size,
            grid_cols,
            grid_rows,
            grid,
            points,
            active_lists,
            total_active_area,
            b0,
            rng,
        })
    }

    fn generate(&mut self) -> PyResult<Vec<(f64, f64)>> {
         while self.total_active_area > 1e-9 {
             if let Some((square, level_idx)) = self.choose_active_square() {
                 let sq_size = self.get_square_size(level_idx);

                 // Clamp random point generation within the domain boundary
                 let px_max = (square.x + sq_size).min(self.width);
                 let py_max = (square.y + sq_size).min(self.height);
                 let px_min = square.x;
                 let py_min = square.y;

                 // Ensure min is not greater than max (can happen for squares partially outside)
                 if px_min >= px_max || py_min >= py_max {
                      continue; 
                 }

                 // Use random_range instead of gen_range(deprecated)
                 let candidate_x = self.rng.random_range(px_min..px_max);
                 let candidate_y = self.rng.random_range(py_min..py_max);


                 if self.is_dart_valid(candidate_x, candidate_y) {
                     self.add_point(candidate_x, candidate_y);
                 } else {
                     // Subdivide
                     let child_level = level_idx + 1;
                     let child_size = sq_size / 2.0;

                     self.add_child_square(child_level, square.x, square.y);
                     self.add_child_square(child_level, square.x + child_size, square.y);
                     self.add_child_square(child_level, square.x, square.y + child_size);
                     self.add_child_square(child_level, square.x + child_size, square.y + child_size);
                 }

             } else {
                 break; 
             }
         }

         // Convert points to Python-friendly format
         let result = self.points.iter().map(|p| (p.x, p.y)).collect();
         Ok(result)
     }
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "hdt_sampling")] 
fn hdt_sampling_module(m: &Bound<'_, PyModule>) -> PyResult<()> { 
    m.add_class::<HDTSampler>()?; 
    Ok(())
}