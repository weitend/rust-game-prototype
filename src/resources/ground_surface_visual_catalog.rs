use std::path::Path;

use bevy::prelude::{Resource, Vec2};

use crate::components::ground_surface::GroundSurfaceKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundVisualSeason {
    Temperate,
    Snow,
}

#[derive(Clone, Copy, Debug)]
pub struct GroundTextureSet {
    pub base_color_path: &'static str,
    pub normal_map_path: Option<&'static str>,
    pub metallic_roughness_map_path: Option<&'static str>,
    pub occlusion_map_path: Option<&'static str>,
    pub uv_tiling: Vec2,
    pub perceptual_roughness: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct GroundSurfaceVisualCatalog {
    pub season: GroundVisualSeason,
}

impl GroundSurfaceVisualCatalog {
    pub const fn with_season(season: GroundVisualSeason) -> Self {
        Self { season }
    }

    pub fn terrain_texture_set_for(&self, kind: GroundSurfaceKind) -> Option<GroundTextureSet> {
        if matches!(self.season, GroundVisualSeason::Snow) {
            if let Some(set) = snow_texture_set_for(kind).filter(texture_set_assets_exist) {
                return Some(set);
            }
        }

        temperate_texture_set_for(kind).filter(texture_set_assets_exist)
    }
}

impl Default for GroundSurfaceVisualCatalog {
    fn default() -> Self {
        Self::with_season(GroundVisualSeason::Temperate)
    }
}

fn temperate_texture_set_for(kind: GroundSurfaceKind) -> Option<GroundTextureSet> {
    match kind {
        GroundSurfaceKind::Default | GroundSurfaceKind::Grass | GroundSurfaceKind::Mud => {
            Some(dirt_coast_sand_rocks_set())
        }
        GroundSurfaceKind::Rock | GroundSurfaceKind::Asphalt => None,
    }
}

fn snow_texture_set_for(kind: GroundSurfaceKind) -> Option<GroundTextureSet> {
    match kind {
        GroundSurfaceKind::Default | GroundSurfaceKind::Grass | GroundSurfaceKind::Mud => {
            Some(snow_default_set())
        }
        GroundSurfaceKind::Rock | GroundSurfaceKind::Asphalt => None,
    }
}

fn dirt_coast_sand_rocks_set() -> GroundTextureSet {
    GroundTextureSet {
        base_color_path: "textures/ground/dirt/coast_sand_rocks_02/coast_sand_rocks_02_diff_4k.jpg",
        normal_map_path: Some(
            "textures/ground/dirt/coast_sand_rocks_02/coast_sand_rocks_02_nor_gl_4k.png",
        ),
        metallic_roughness_map_path: Some(
            "textures/ground/dirt/coast_sand_rocks_02/coast_sand_rocks_02_arm_4k.jpg",
        ),
        occlusion_map_path: Some(
            "textures/ground/dirt/coast_sand_rocks_02/coast_sand_rocks_02_ao_4k.jpg",
        ),
        uv_tiling: Vec2::splat(8.0),
        perceptual_roughness: 0.94,
    }
}

fn snow_default_set() -> GroundTextureSet {
    GroundTextureSet {
        base_color_path: "textures/ground/snow/default/default_diff_4k.jpg",
        normal_map_path: Some("textures/ground/snow/default/default_nor_gl_4k.png"),
        metallic_roughness_map_path: Some("textures/ground/snow/default/default_arm_4k.jpg"),
        occlusion_map_path: None,
        uv_tiling: Vec2::splat(8.0),
        perceptual_roughness: 0.96,
    }
}

fn texture_set_assets_exist(set: &GroundTextureSet) -> bool {
    asset_exists(set.base_color_path)
        && optional_asset_exists(set.normal_map_path)
        && optional_asset_exists(set.metallic_roughness_map_path)
        && optional_asset_exists(set.occlusion_map_path)
}

fn optional_asset_exists(path: Option<&'static str>) -> bool {
    path.is_none_or(asset_exists)
}

fn asset_exists(asset_path: &'static str) -> bool {
    let assets_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    assets_root.join(asset_path).is_file()
}
