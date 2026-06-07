//! Stack-local scratch buffers for zero-allocation ray/polygon tests.

pub struct FootprintScratch {
    pub xs: [f64; 64],
    pub ys: [f64; 64],
    pub len: usize,
}

impl FootprintScratch {
    pub fn load_vertices(&mut self, verts: &[horizon_geometry::ArchivedPoint3]) -> bool {
        let count = verts.len();
        if count < 3 || count > 64 {
            return false;
        }
        for (i, v) in verts.iter().enumerate() {
            self.xs[i] = v.x;
            self.ys[i] = v.y;
        }
        self.len = count;
        true
    }

    pub fn xs(&self) -> &[f64] {
        &self.xs[..self.len]
    }

    pub fn ys(&self) -> &[f64] {
        &self.ys[..self.len]
    }
}

impl Default for FootprintScratch {
    fn default() -> Self {
        Self {
            xs: [0.0; 64],
            ys: [0.0; 64],
            len: 0,
        }
    }
}
