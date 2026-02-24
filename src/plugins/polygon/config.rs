use bevy::prelude::{Resource, Vec3};

#[derive(Resource, Clone, Debug)]
pub struct PolygonConfig {
    pub module_grid: usize,
    pub module_size: f32,
    pub module_gap: f32,
    pub platform_height: f32,
    pub tile_size: f32,
}

impl Default for PolygonConfig {
    fn default() -> Self {
        Self {
            module_grid: 10,
            module_size: 20.0,
            module_gap: 8.0,
            platform_height: 0.2,
            tile_size: 3.0,
        }
    }
}

impl PolygonConfig {
    pub fn sanitized(&self) -> Self {
        Self {
            module_grid: self.module_grid.max(1),
            module_size: self.module_size.max(1.0),
            module_gap: self.module_gap.max(0.0),
            platform_height: self.platform_height.max(0.05),
            tile_size: self.tile_size.max(0.25),
        }
    }

    pub fn module_pitch(&self) -> f32 {
        self.module_size + self.module_gap
    }

    pub fn span_for_modules(&self, modules: usize) -> f32 {
        if modules == 0 {
            return 0.0;
        }

        self.module_size * modules as f32 + self.module_gap * modules.saturating_sub(1) as f32
    }

    pub fn platform_span(&self) -> f32 {
        self.span_for_modules(self.module_grid)
    }

    pub fn platform_size(&self) -> Vec3 {
        let span = self.platform_span();
        Vec3::new(span, self.platform_height, span)
    }

    pub fn module_center(&self, col: usize, row: usize) -> Vec3 {
        let span = self.platform_span();
        let min_x = -0.5 * span + 0.5 * self.module_size;
        let min_z = -0.5 * span + 0.5 * self.module_size;
        let pitch = self.module_pitch();

        Vec3::new(min_x + col as f32 * pitch, 0.0, min_z + row as f32 * pitch)
    }
}
