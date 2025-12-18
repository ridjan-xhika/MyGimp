use crate::canvas::Canvas;

const MAX_HISTORY: usize = 50;

pub struct HistoryState {
    pub pixels: Vec<u8>,
    pub drawing_layer: Vec<u8>,
    pub stride: usize,
    pub width: u32,
    pub height: u32,
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

    pub fn push(&mut self, canvas: &Canvas) {
        // Remove any states after current (if user made a change after undoing)
        self.states.truncate(self.current + 1);

        // Limit history size
        if self.states.len() >= MAX_HISTORY {
            self.states.remove(0);
        } else {
            self.current += 1;
        }

        self.states.push(HistoryState {
            pixels: canvas.pixels.clone(),
            drawing_layer: canvas.drawing_layer.clone(),
            stride: canvas.stride,
            width: canvas.width,
            height: canvas.height,
        });
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
}
