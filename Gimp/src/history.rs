use crate::canvas::Canvas;
use serde::{Deserialize, Serialize};

const MAX_HISTORY: usize = 50;
const THUMBNAIL_SIZE: u32 = 64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryState {
    pub pixels: Vec<u8>,
    pub drawing_layer: Vec<u8>,
    pub stride: usize,
    pub width: u32,
    pub height: u32,
    pub thumbnail: Vec<u8>, // Cached thumbnail (64x64 RGBA)
    pub description: String,
    pub timestamp: u64,
}

impl HistoryState {
    fn generate_thumbnail(&self) -> Vec<u8> {
        let mut thumb = vec![255u8; (THUMBNAIL_SIZE * THUMBNAIL_SIZE * 4) as usize];

        if self.width == 0 || self.height == 0 {
            return thumb;
        }

        let scale_x = self.width as f32 / THUMBNAIL_SIZE as f32;
        let scale_y = self.height as f32 / THUMBNAIL_SIZE as f32;

        for ty in 0..THUMBNAIL_SIZE {
            for tx in 0..THUMBNAIL_SIZE {
                let src_x = (tx as f32 * scale_x) as u32;
                let src_y = (ty as f32 * scale_y) as u32;

                if src_x < self.width && src_y < self.height {
                    let src_idx = ((src_y * self.stride as u32 + src_x) * 4) as usize;
                    if src_idx + 3 < self.pixels.len() {
                        let dst_idx = ((ty * THUMBNAIL_SIZE + tx) * 4) as usize;
                        thumb[dst_idx] = self.pixels[src_idx];
                        thumb[dst_idx + 1] = self.pixels[src_idx + 1];
                        thumb[dst_idx + 2] = self.pixels[src_idx + 2];
                        thumb[dst_idx + 3] = self.pixels[src_idx + 3];
                    }
                }
            }
        }

        thumb
    }
}

pub struct History {
    states: Vec<HistoryState>,
    current: usize,
}

impl History {
    pub fn new() -> Self {
        History {
            states: Vec::new(),
            current: 0,
        }
    }

    pub fn push(&mut self, canvas: &Canvas, description: String) {
        // Remove any states after current (if user made a change after undoing)
        self.states.truncate(self.current + 1);

        // Limit history size
        if self.states.len() >= MAX_HISTORY {
            self.states.remove(0);
        } else {
            self.current += 1;
        }

        let state = HistoryState {
            pixels: canvas.pixels.clone(),
            drawing_layer: canvas.drawing_layer.clone(),
            stride: canvas.stride,
            width: canvas.width,
            height: canvas.height,
            thumbnail: vec![], // Will be generated lazily
            description,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        self.states.push(state);
    }

    pub fn get_thumbnail(&mut self, index: usize) -> Option<&[u8]> {
        if index < self.states.len() {
            if self.states[index].thumbnail.is_empty() {
                self.states[index].thumbnail = self.states[index].generate_thumbnail();
            }
            Some(&self.states[index].thumbnail)
        } else {
            None
        }
    }

    pub fn get_state_description(&self, index: usize) -> Option<&str> {
        if index < self.states.len() {
            Some(&self.states[index].description)
        } else {
            None
        }
    }

    pub fn history_count(&self) -> usize {
        self.states.len()
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn undo(&mut self, canvas: &mut Canvas) -> bool {
        if self.current > 0 {
            self.current -= 1;
            let state = &self.states[self.current];
            canvas.pixels = state.pixels.clone();
            canvas.drawing_layer = state.drawing_layer.clone();
            canvas.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, canvas: &mut Canvas) -> bool {
        if self.current + 1 < self.states.len() {
            self.current += 1;
            let state = &self.states[self.current];
            canvas.pixels = state.pixels.clone();
            canvas.drawing_layer = state.drawing_layer.clone();
            canvas.dirty = true;
            true
        } else {
            false
        }
    }

    /// Jump to a specific history state by index
    pub fn jump_to(&mut self, index: usize, canvas: &mut Canvas) -> bool {
        if index < self.states.len() {
            self.current = index;
            let state = &self.states[self.current];
            canvas.pixels = state.pixels.clone();
            canvas.drawing_layer = state.drawing_layer.clone();
            canvas.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn restore(&self, canvas: &mut Canvas, state: &HistoryState) {
        canvas.pixels = state.pixels.clone();
        canvas.drawing_layer = state.drawing_layer.clone();
        canvas.dirty = true;
    }

    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current + 1 < self.states.len()
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.states
            .iter()
            .map(|s| s.pixels.len() + s.drawing_layer.len() + s.thumbnail.len())
            .sum()
    }
}
