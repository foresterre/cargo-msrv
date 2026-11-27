pub struct ScopeStack {
    frames: Vec<Frame>,
}

struct Frame {
    id: usize,
    label: &'static str,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn push(&mut self, id: usize, label: &'static str) {
        self.frames.push(Frame { id, label });
    }

    pub fn pop(&mut self, id: usize) {
        match self.frames.iter().rposition(|frame| frame.id == id) {
            Some(at) => {
                self.close_to(at + 1);
                self.frames.pop();
            }
            None => tracing::warn!(id, "closing a scope which was never opened"),
        }
    }

    pub fn close_to(&mut self, depth: usize) {
        if depth >= self.frames.len() {
            return;
        }

        for frame in self.frames.drain(depth..) {
            tracing::warn!(id = frame.id, label = frame.label, "scope was never closed");
        }
    }
}
