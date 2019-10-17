use crate::projectile::{Projectile, Step};
use direction::CardinalDirection;
pub use ecs::Entity;
use ecs::{ecs_components, ComponentTable, Ecs};
use grid_2d::{Coord, Grid, Size};
use line_2d::LineSegment;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Bullet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Layer {
    Floor,
    Feature,
    Character,
    Projectile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialLayer {
    Floor,
    Feature,
    Character,
}

impl SpatialLayer {
    fn to_layer(self) -> Layer {
        match self {
            Self::Floor => Layer::Floor,
            Self::Feature => Layer::Feature,
            Self::Character => Layer::Character,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub coord: Coord,
    pub spatial_layer: SpatialLayer,
}

impl Location {
    fn new(coord: Coord, spatial_layer: SpatialLayer) -> Self {
        Self { coord, spatial_layer }
    }
}

ecs_components! {
    components {
        location: Location,
        tile: Tile,
        opacity: u8,
        solid: (),
        projectile: Projectile,
    }
}
use components::Components;

#[derive(Debug)]
struct OccupiedBy(Entity);

#[derive(Debug, Serialize, Deserialize)]
struct SpatialCell {
    floor: Option<Entity>,
    feature: Option<Entity>,
    character: Option<Entity>,
}

impl Default for SpatialCell {
    fn default() -> Self {
        Self {
            floor: None,
            feature: None,
            character: None,
        }
    }
}

impl SpatialCell {
    fn select_field_mut(&mut self, spatial_layer: SpatialLayer) -> &mut Option<Entity> {
        match spatial_layer {
            SpatialLayer::Character => &mut self.character,
            SpatialLayer::Feature => &mut self.feature,
            SpatialLayer::Floor => &mut self.floor,
        }
    }
    fn insert(&mut self, entity: Entity, spatial_layer: SpatialLayer) -> Result<(), OccupiedBy> {
        let layer_field = self.select_field_mut(spatial_layer);
        if let Some(&occupant) = layer_field.as_ref() {
            Err(OccupiedBy(occupant))
        } else {
            *layer_field = Some(entity);
            Ok(())
        }
    }
    fn clear(&mut self, spatial_layer: SpatialLayer) -> Option<Entity> {
        self.select_field_mut(spatial_layer).take()
    }
}

fn location_insert(
    entity: Entity,
    location: Location,
    location_component: &mut ComponentTable<Location>,
    spatial_grid: &mut Grid<SpatialCell>,
) -> Result<(), OccupiedBy> {
    let cell = spatial_grid.get_checked_mut(location.coord);
    cell.insert(entity, location.spatial_layer)?;
    if let Some(original_location) = location_component.insert(entity, location) {
        let original_cell = spatial_grid.get_checked_mut(original_location.coord);
        let should_match_entity = original_cell.clear(original_location.spatial_layer);
        debug_assert_eq!(
            should_match_entity,
            Some(entity),
            "Current location of entity doesn't contain entity in spatial grid"
        );
    }
    Ok(())
}

fn is_solid_feature_at_coord(
    coord: Coord,
    solid_component: &ComponentTable<()>,
    spatial_grid: &Grid<SpatialCell>,
) -> bool {
    let cell = spatial_grid.get_checked(coord);
    if let Some(feature) = cell.feature {
        solid_component.contains(feature)
    } else {
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    ecs: Ecs<Components>,
    spatial_grid: Grid<SpatialCell>,
    entity_buffer: Vec<Entity>,
}

impl World {
    pub fn new(size: Size) -> Self {
        let ecs = Ecs::new();
        let spatial_grid = Grid::new_default(size);
        let entity_buffer = Vec::new();
        Self {
            ecs,
            spatial_grid,
            entity_buffer,
        }
    }
    pub fn spawn_player(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, SpatialLayer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Player);
        entity
    }
    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, SpatialLayer::Floor),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Floor);
        entity
    }
    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, SpatialLayer::Feature),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Wall);
        self.ecs.components.solid.insert(entity, ());
        self.ecs.components.opacity.insert(entity, 255);
        entity
    }
    pub fn character_walk_in_direction(&mut self, entity: Entity, direction: CardinalDirection) {
        let current_location = self.ecs.components.location.get_mut(entity).unwrap();
        debug_assert_eq!(current_location.spatial_layer, SpatialLayer::Character);
        let target_coord = current_location.coord + direction.coord();
        if is_solid_feature_at_coord(target_coord, &self.ecs.components.solid, &self.spatial_grid) {
            return;
        }
        let target_location = Location::new(target_coord, SpatialLayer::Character);
        if let Err(OccupiedBy(_occupant)) = location_insert(
            entity,
            target_location,
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        ) {
            // TODO melee
        }
    }
    pub fn character_fire_bullet(&mut self, character: Entity, target: Coord) {
        let character_coord = self.ecs.components.location.get(character).unwrap().coord;
        if character_coord == target {
            return;
        }
        let bullet_entity = self.ecs.create();
        let path = LineSegment::new(character_coord, target);
        self.ecs
            .components
            .projectile
            .insert(bullet_entity, Projectile::new(path, Duration::from_millis(20)));
        self.ecs.components.tile.insert(bullet_entity, Tile::Bullet);
    }
    pub fn entity_coord(&self, entity: Entity) -> Coord {
        self.ecs.components.location.get(entity).unwrap().coord
    }
    pub fn size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn has_pending_animation(&self) -> bool {
        !self.ecs.components.projectile.is_empty()
    }
    pub fn animation_tick<R: Rng>(&mut self, _rng: &mut R) {
        let to_remove = &mut self.entity_buffer;
        for (entity, projectile) in self.ecs.components.projectile.iter_mut() {
            for step in projectile.frame_iter() {
                match step {
                    Step::AtDestination => {
                        to_remove.push(entity);
                        break;
                    }
                    Step::MoveTo(coord) => {
                        if is_solid_feature_at_coord(coord, &self.ecs.components.solid, &self.spatial_grid) {
                            to_remove.push(entity);
                            break;
                        }
                    }
                }
            }
        }
        for entity in to_remove.drain(..) {
            self.ecs.remove(entity);
        }
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        let tile_component = &self.ecs.components.tile;
        let location_component = &self.ecs.components.location;
        let projectile_component = &self.ecs.components.projectile;
        tile_component.iter().filter_map(move |(entity, &tile)| {
            if let Some(location) = location_component.get(entity) {
                Some(ToRenderEntity {
                    coord: location.coord,
                    layer: location.spatial_layer.to_layer(),
                    tile: tile,
                })
            } else if let Some(projectile) = projectile_component.get(entity) {
                Some(ToRenderEntity {
                    coord: projectile.coord(),
                    layer: Layer::Projectile,
                    tile,
                })
            } else {
                None
            }
        })
    }
}

pub struct ToRenderEntity {
    pub coord: Coord,
    pub layer: Layer,
    pub tile: Tile,
}
