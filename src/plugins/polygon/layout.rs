#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionKind {
    Reserved,
    ControlHub,
    MovementCalibration,
    JumpAutostepLab,
    CollisionTorture,
    HitscanRange,
    CoverPeek,
    DamageTeamSandbox,
    VerticalCombat,
    PerformanceStress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionBounds {
    pub min_col: usize,
    pub max_col: usize,
    pub min_row: usize,
    pub max_row: usize,
}

impl SectionBounds {
    pub fn width_modules(self) -> usize {
        self.max_col - self.min_col + 1
    }

    pub fn height_modules(self) -> usize {
        self.max_row - self.min_row + 1
    }
}

#[derive(Clone, Debug)]
pub struct SectionLayout {
    grid: usize,
    cells: Vec<SectionKind>,
}

impl SectionLayout {
    pub fn default_for_grid(grid: usize) -> Self {
        let grid = grid.max(1);
        let mut cells = vec![SectionKind::Reserved; grid * grid];

        let mut set_row = |row: usize, kind: SectionKind| {
            for col in 0..grid {
                cells[row * grid + col] = kind;
            }
        };

        let set_range = |cells: &mut [SectionKind],
                         row: usize,
                         col_start: usize,
                         col_end_exclusive: usize,
                         kind: SectionKind| {
            for col in col_start..col_end_exclusive {
                cells[row * grid + col] = kind;
            }
        };

        let top = 0;
        set_row(top, SectionKind::MovementCalibration);

        if grid > 1 {
            set_row(1, SectionKind::HitscanRange);
        }

        if grid > 2 {
            let last = grid - 1;
            set_row(last, SectionKind::PerformanceStress);
        }

        if grid > 3 {
            let jump_row = grid - 2;
            set_row(jump_row, SectionKind::JumpAutostepLab);
        }

        let center_left = if grid > 1 { (grid - 1) / 2 } else { 0 };
        let center_right = (center_left + 1).min(grid - 1);
        let hub_top = if grid > 2 {
            (grid / 2).saturating_sub(1)
        } else {
            0
        };
        let hub_bottom = (hub_top + 1).min(grid - 1);

        for row in hub_top..=hub_bottom {
            for col in center_left..=center_right {
                cells[row * grid + col] = SectionKind::ControlHub;
            }
        }

        if center_left > 0 {
            set_range(
                &mut cells,
                hub_top,
                0,
                center_left,
                SectionKind::CollisionTorture,
            );
            set_range(
                &mut cells,
                hub_bottom,
                0,
                center_left,
                SectionKind::DamageTeamSandbox,
            );
        }

        if center_right + 1 < grid {
            set_range(
                &mut cells,
                hub_top,
                center_right + 1,
                grid,
                SectionKind::CoverPeek,
            );
            set_range(
                &mut cells,
                hub_bottom,
                center_right + 1,
                grid,
                SectionKind::VerticalCombat,
            );
        }

        Self { grid, cells }
    }

    pub fn grid(&self) -> usize {
        self.grid
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, SectionKind)> + '_ {
        self.cells
            .iter()
            .copied()
            .enumerate()
            .map(move |(index, section)| {
                let row = index / self.grid;
                let col = index % self.grid;
                (col, row, section)
            })
    }

    pub fn bounds_of(&self, target: SectionKind) -> Option<SectionBounds> {
        let mut found = false;
        let mut min_col = usize::MAX;
        let mut max_col = 0;
        let mut min_row = usize::MAX;
        let mut max_row = 0;

        for (col, row, section) in self.iter() {
            if section != target {
                continue;
            }

            found = true;
            min_col = min_col.min(col);
            max_col = max_col.max(col);
            min_row = min_row.min(row);
            max_row = max_row.max(row);
        }

        if !found {
            return None;
        }

        Some(SectionBounds {
            min_col,
            max_col,
            min_row,
            max_row,
        })
    }
}
