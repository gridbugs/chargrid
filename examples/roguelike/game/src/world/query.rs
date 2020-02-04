use crate::world::{data::Tile, World};
use ecs::Entity;
use grid_2d::Coord;
use line_2d::LineSegment;

pub trait WorldQuery {
    fn is_solid_feature_at_coord(&self, coord: Coord) -> bool;
    fn is_solid_feature_in_line_segment(&self, line_segment: LineSegment) -> bool;
    fn is_wall_at_coord(&self, coord: Coord) -> bool;
    fn is_npc_at_coord(&self, coord: Coord) -> bool;
    fn get_opacity_at_coord(&self, coord: Coord) -> u8;
    fn get_character_at_coord(&self, coord: Coord) -> Option<Entity>;
}

impl WorldQuery for World {
    fn is_solid_feature_at_coord(&self, coord: Coord) -> bool {
        let cell = self.spatial_grid.get_checked(coord);
        if let Some(feature) = cell.feature {
            self.ecs.components.solid.contains(feature)
        } else {
            false
        }
    }

    fn is_solid_feature_in_line_segment(&self, line_segment: LineSegment) -> bool {
        for coord in line_segment.iter() {
            if self.is_solid_feature_at_coord(coord) {
                return true;
            }
        }
        false
    }

    fn is_wall_at_coord(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_grid.get(coord) {
            if let Some(entity) = spatial_cell.feature {
                self.ecs.components.tile.get(entity) == Some(&Tile::Wall)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_npc_at_coord(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_grid.get(coord) {
            if let Some(entity) = spatial_cell.character {
                self.ecs.components.npc.contains(entity)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_opacity_at_coord(&self, coord: Coord) -> u8 {
        self.spatial_grid
            .get(coord)
            .and_then(|c| c.feature)
            .and_then(|e| self.ecs.components.opacity.get(e).cloned())
            .unwrap_or(0)
    }

    fn get_character_at_coord(&self, coord: Coord) -> Option<Entity> {
        self.spatial_grid.get(coord).and_then(|cell| cell.character)
    }
}
