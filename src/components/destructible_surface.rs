use std::collections::{HashSet, VecDeque};

use bevy::{math::Vec3, prelude::Component};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SurfaceCellKey {
    x: i32,
    y: i32,
    z: i32,
}

impl SurfaceCellKey {
    fn from_local_point(local_point: Vec3, cell_size: f32) -> Self {
        let inv_cell = 1.0 / cell_size;
        let scaled = local_point * inv_cell;

        Self {
            x: scaled.x.floor() as i32,
            y: scaled.y.floor() as i32,
            z: scaled.z.floor() as i32,
        }
    }
}

#[derive(Component, Debug)]
pub struct DestructibleSurface {
    pub cell_size: f32,
    pub max_marks: usize,
    marked_cells: HashSet<SurfaceCellKey>,
    mark_order: VecDeque<SurfaceCellKey>,
}

impl Default for DestructibleSurface {
    fn default() -> Self {
        Self {
            cell_size: 0.14,
            max_marks: 96,
            marked_cells: HashSet::new(),
            mark_order: VecDeque::new(),
        }
    }
}

impl DestructibleSurface {
    pub fn try_mark(&mut self, local_point: Vec3) -> bool {
        if self.cell_size <= f32::EPSILON {
            return false;
        }

        if self.max_marks == 0 {
            return false;
        }

        let cell = SurfaceCellKey::from_local_point(local_point, self.cell_size);
        if self.marked_cells.contains(&cell) {
            return false;
        }

        while self.marked_cells.len() >= self.max_marks {
            let Some(oldest) = self.mark_order.pop_front() else {
                self.marked_cells.clear();
                break;
            };
            self.marked_cells.remove(&oldest);
        }

        let inserted = self.marked_cells.insert(cell);
        if inserted {
            self.mark_order.push_back(cell);
        }
        inserted
    }
}
