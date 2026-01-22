use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    Replace,
    Add,
    Subtract,
    Intersect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionType {
    Rectangle,
    Ellipse,
    Lasso,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Selection {
    pub mask: Vec<bool>, // Pixel-by-pixel selection mask
    pub width: u32,
    pub height: u32,
    pub bounds: Option<(u32, u32, u32, u32)>, // (x, y, w, h) for optimization
}

impl Selection {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            mask: vec![false; size],
            width,
            height,
            bounds: None,
        }
    }

    pub fn clear(&mut self) {
        self.mask.iter_mut().for_each(|m| *m = false);
        self.bounds = None;
    }

    pub fn select_all(&mut self) {
        self.mask.iter_mut().for_each(|m| *m = true);
        self.bounds = Some((0, 0, self.width, self.height));
    }

    pub fn invert(&mut self) {
        self.mask.iter_mut().for_each(|m| *m = !*m);
    }

    pub fn is_selected(&self, x: u32, y: u32) -> bool {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.mask[idx]
        } else {
            false
        }
    }

    pub fn set_selected(&mut self, x: u32, y: u32, selected: bool) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.mask[idx] = selected;
        }
    }

    /// Create a rectangular selection
    pub fn select_rectangle(
        &mut self,
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        mode: SelectionMode,
    ) {
        let x_min = x1.min(x2).max(0).min(self.width - 1);
        let x_max = x1.max(x2).max(0).min(self.width - 1);
        let y_min = y1.min(y2).max(0).min(self.height - 1);
        let y_max = y1.max(y2).max(0).min(self.height - 1);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let idx = (y * self.width + x) as usize;
                match mode {
                    SelectionMode::Replace => self.mask[idx] = true,
                    SelectionMode::Add => self.mask[idx] = true,
                    SelectionMode::Subtract => self.mask[idx] = false,
                    SelectionMode::Intersect => {
                        self.mask[idx] = self.mask[idx] && true;
                    }
                }
            }
        }

        self.bounds = Some((x_min, y_min, x_max - x_min + 1, y_max - y_min + 1));
    }

    /// Create an elliptical selection
    pub fn select_ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        mode: SelectionMode,
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist_sq = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry);

                if dist_sq <= 1.0 {
                    let idx = (y * self.width + x) as usize;
                    match mode {
                        SelectionMode::Replace | SelectionMode::Add => self.mask[idx] = true,
                        SelectionMode::Subtract => self.mask[idx] = false,
                        SelectionMode::Intersect => {
                            self.mask[idx] = self.mask[idx] && true;
                        }
                    }
                }
            }
        }
    }

    /// Create a lasso (freehand) selection from a list of points
    pub fn select_lasso(&mut self, points: &[(f32, f32)], mode: SelectionMode) {
        if points.len() < 3 {
            return;
        }

        // Fill polygon using scan-line algorithm
        let mut fill_lines: Vec<Vec<(i32, i32)>> = vec![vec![]; self.height as usize];

        // Add edges to fill_lines
        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];

            self.add_edge_to_fill(&mut fill_lines, p1, p2);
        }

        // Scan fill
        for y in 0..self.height {
            let mut intervals = fill_lines[y as usize].clone();
            intervals.sort();

            let mut inside = false;
            for x in 0..self.width {
                for &(x1, x2) in &intervals {
                    if x >= x1 as u32 && x <= x2 as u32 {
                        inside = !inside;
                    }
                }

                if inside {
                    let idx = (y * self.width + x) as usize;
                    match mode {
                        SelectionMode::Replace | SelectionMode::Add => self.mask[idx] = true,
                        SelectionMode::Subtract => self.mask[idx] = false,
                        SelectionMode::Intersect => {
                            self.mask[idx] = self.mask[idx] && true;
                        }
                    }
                }
            }
        }
    }

    fn add_edge_to_fill(
        &self,
        fill_lines: &mut [Vec<(i32, i32)>],
        p1: (f32, f32),
        p2: (f32, f32),
    ) {
        let (x1, y1) = (p1.0 as i32, p1.1 as i32);
        let (x2, y2) = (p2.0 as i32, p2.1 as i32);

        if y1 == y2 {
            return; // Horizontal edge, ignore
        }

        let (y_min, y_max) = if y1 < y2 {
            (y1, y2)
        } else {
            (y2, y1)
        };

        let dy = (y2 - y1) as f32;
        let dx = (x2 - x1) as f32;

        for y in y_min..=y_max {
            if y < 0 || y >= self.height as i32 {
                continue;
            }

            let t = (y - y1) as f32 / dy;
            let x_intersect = (x1 as f32 + t * dx) as i32;

            fill_lines[y as usize].push((x_intersect, x_intersect));
        }
    }

    /// Check if there's an active selection
    pub fn is_active(&self) -> bool {
        self.mask.iter().any(|&m| m)
    }

    /// Count selected pixels
    pub fn count_selected(&self) -> usize {
        self.mask.iter().filter(|&&m| m).count()
    }
}
