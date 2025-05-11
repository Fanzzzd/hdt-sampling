use pyo3::prelude::*;
use rand::{Rng, SeedableRng};
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
pub struct HDTSampler {
    width: f64,
    height: f64,
    r_sq: f64,
    grid_cell_size: f64,
    grid_cols: usize,
    grid_rows: usize,
    grid: Vec<GridCell>,
    b0: f64,
    active_lists: Vec<Vec<Square>>,
    total_active_area: f64,
    points: Vec<Point>,
    rng: StdRng,
}

impl HDTSampler {
    #[inline]
    fn grid_coords(&self, x: f64, y: f64) -> (usize, usize) {
        let gx = (x / self.grid_cell_size).floor() as usize;
        let gy = (y / self.grid_cell_size).floor() as usize;
        (
            gx.min(self.grid_cols - 1),
            gy.min(self.grid_rows - 1),
        )
    }

    #[inline]
    fn get_square_size(&self, level: usize) -> f64 {
        self.b0 / 2.0f64.powi(level as i32)
    }

    /// iterate over the 3×3 neighborhood of a cell
    fn neighborhood_indices<'a>(
        &'a self,
        gx: usize,
        gy: usize,
    ) -> impl Iterator<Item = usize> + 'a {
        let x0 = max(0, gx as i32 - 1) as usize;
        let x1 = min(self.grid_cols - 1, gx + 1);
        let y0 = max(0, gy as i32 - 1) as usize;
        let y1 = min(self.grid_rows - 1, gy + 1);
        (x0..=x1).flat_map(move |i| (y0..=y1).map(move |j| i + j * self.grid_cols))
    }

    fn is_dart_valid(&self, x: f64, y: f64) -> bool {
        let (gx, gy) = self.grid_coords(x, y);
        for cell_idx in self.neighborhood_indices(gx, gy) {
            for &pt_idx in &self.grid[cell_idx].indices {
                let p = &self.points[pt_idx];
                let dx = x - p.x;
                let dy = y - p.y;
                if dx * dx + dy * dy < self.r_sq {
                    return false;
                }
            }
        }
        true
    }

    fn farthest_corner_dist_sq(&self, square: &Square, px: f64, py: f64) -> f64 {
        let s = self.get_square_size(square.level);
        let cx = square.x + s / 2.0;
        let cy = square.y + s / 2.0;
        let dx = (cx - px).abs() + s / 2.0;
        let dy = (cy - py).abs() + s / 2.0;
        dx * dx + dy * dy
    }

    fn is_square_covered(&self, square: &Square) -> bool {
        let s = self.get_square_size(square.level);
        let cx = square.x + s / 2.0;
        let cy = square.y + s / 2.0;
        let (gx, gy) = self.grid_coords(cx, cy);
        for cell_idx in self.neighborhood_indices(gx, gy) {
            for &pt_idx in &self.grid[cell_idx].indices {
                let p = &self.points[pt_idx];
                if self.farthest_corner_dist_sq(square, p.x, p.y) < self.r_sq {
                    return true;
                }
            }
        }
        false
    }

    fn add_point(&mut self, x: f64, y: f64) {
        let idx = self.points.len();
        self.points.push(Point { x, y });
        let (gx, gy) = self.grid_coords(x, y);
        let cell_idx = gx + gy * self.grid_cols;
        self.grid[cell_idx].indices.push(idx);
    }

    fn ensure_level(&mut self, level: usize) {
        while self.active_lists.len() <= level {
            self.active_lists.push(Vec::new());
        }
    }

    fn add_child_square(&mut self, level: usize, x: f64, y: f64) {
        if x >= self.width || y >= self.height {
            return;
        }
        let sq = Square { level, x, y };
        if self.is_square_covered(&sq) {
            return;
        }
        self.ensure_level(level);
        self.active_lists[level].push(sq);
        let s = self.get_square_size(level);
        self.total_active_area += s * s;
    }

    fn choose_active_square(&mut self) -> Option<(Square, usize)> {
        if self.total_active_area <= f64::EPSILON {
            return None;
        }
        let target = self.rng.random_range(0.0..self.total_active_area);
        let mut acc = 0.0;
        for level in 0..self.active_lists.len() {
            let s = self.get_square_size(level);
            let area = s * s;
            let list = &mut self.active_lists[level];
            if list.is_empty() {
                continue;
            }
            let level_area = area * list.len() as f64;
            if acc + level_area > target {
                let idx_in_level = ((target - acc) / area).floor() as usize;
                let chosen = list.swap_remove(idx_in_level);
                self.total_active_area -= area;
                return Some((chosen, level));
            }
            acc += level_area;
        }
        None
    }
}

#[pymethods]
impl HDTSampler {
    #[new]
    #[pyo3(signature = (width, height, r, seed=None))]
    pub fn new(width: f64, height: f64, r: f64, seed: Option<u64>) -> PyResult<Self> {
        if r <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err("r must be positive"));
        }
        let r_sq = r * r;
        let grid_cell_size = r;
        let grid_cols = ((width / grid_cell_size).ceil() as usize).max(1);
        let grid_rows = ((height / grid_cell_size).ceil() as usize).max(1);
        let grid = vec![GridCell { indices: Vec::new() }; grid_cols * grid_rows];

        let b0 = r / std::f64::consts::SQRT_2;

        let mut active_lists: Vec<Vec<Square>> = vec![Vec::new()];
        let mut total_active_area = 0.0;

        // initialize base‑level squares so they tile the domain
        let cols_base = ((width / b0).ceil() as usize).max(1);
        let rows_base = ((height / b0).ceil() as usize).max(1);
        for i in 0..cols_base {
            for j in 0..rows_base {
                let x = i as f64 * b0;
                let y = j as f64 * b0;
                if x < width && y < height {
                    active_lists[0].push(Square { level: 0, x, y });
                    total_active_area += b0 * b0;
                }
            }
        }

        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_os_rng(),
        };

        Ok(Self {
            width,
            height,
            r_sq,
            grid_cell_size,
            grid_cols,
            grid_rows,
            grid,
            b0,
            active_lists,
            total_active_area,
            points: Vec::new(),
            rng,
        })
    }

    pub fn generate(&mut self) -> PyResult<Vec<(f64, f64)>> {
        while let Some((square, level)) = self.choose_active_square() {
            if self.is_square_covered(&square) {
                continue;
            }
            let s = self.get_square_size(level);
            let px = self.rng.random_range(square.x..square.x + s);
            let py = self.rng.random_range(square.y..square.y + s);
            if self.is_dart_valid(px, py) {
                self.add_point(px, py);
            } else {
                let child_level = level + 1;
                let half = s / 2.0;
                self.add_child_square(child_level, square.x, square.y);
                self.add_child_square(child_level, square.x + half, square.y);
                self.add_child_square(child_level, square.x, square.y + half);
                self.add_child_square(child_level, square.x + half, square.y + half);
            }
        }
        Ok(self.points.iter().map(|p| (p.x, p.y)).collect())
    }
}

/// Python module wrapper
#[pymodule]
#[pyo3(name = "hdt_sampling")]
fn hdt_sampling_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HDTSampler>()?;
    Ok(())
}