use ecs::{ComponentTable, ComponentTableEntries, Entity};
use grid_2d::{Coord, Grid, Size};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Floor,
    Feature,
    Character,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub coord: Coord,
    pub layer: Option<Layer>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpatialCell {
    pub floor: Option<Entity>,
    pub feature: Option<Entity>,
    pub character: Option<Entity>,
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

#[derive(Debug)]
pub struct Spatial {
    location_component: ComponentTable<Location>,
    spatial_grid: Grid<SpatialCell>,
}

impl Spatial {
    pub fn new(size: Size) -> Self {
        let location_component = ComponentTable::default();
        let spatial_grid = Grid::new_default(size);
        Self {
            location_component,
            spatial_grid,
        }
    }
    pub fn grid_size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn get_cell(&self, coord: Coord) -> Option<&SpatialCell> {
        self.spatial_grid.get(coord)
    }
    pub fn get_cell_checked(&self, coord: Coord) -> &SpatialCell {
        self.spatial_grid.get_checked(coord)
    }
    pub fn location(&self, entity: Entity) -> Option<&Location> {
        self.location_component.get(entity)
    }
    pub fn coord(&self, entity: Entity) -> Option<&Coord> {
        self.location(entity).map(|l| &l.coord)
    }
    pub fn insert(&mut self, entity: Entity, location: Location) -> Result<(), OccupiedBy> {
        if let Some(layer) = location.layer {
            let cell = self.spatial_grid.get_checked_mut(location.coord);
            cell.insert(entity, layer)?;
        }
        if let Some(original_location) = self.location_component.insert(entity, location) {
            let original_cell = self.spatial_grid.get_checked_mut(original_location.coord);
            if let Some(original_layer) = original_location.layer {
                let should_match_entity = original_cell.clear(original_layer);
                debug_assert_eq!(
                    should_match_entity,
                    Some(entity),
                    "Current location of entity doesn't contain entity in spatial grid"
                );
            }
        }
        Ok(())
    }
    pub fn update_coord(&mut self, entity: Entity, coord: Coord) -> Result<(), OccupiedBy> {
        if let Some(location) = self.location_component.get_mut(entity) {
            if coord != location.coord {
                if let Some(layer) = location.layer {
                    self.spatial_grid.get_checked_mut(coord).insert(entity, layer)?;
                    let original_cell = self.spatial_grid.get_checked_mut(location.coord);
                    let should_match_entity = original_cell.clear(layer);
                    debug_assert_eq!(
                        should_match_entity,
                        Some(entity),
                        "Current location of entity doesn't contain entity in spatial grid"
                    );
                }
                location.coord = coord;
            }
            Ok(())
        } else {
            self.insert(entity, Location { coord, layer: None })
        }
    }
    pub fn remove(&mut self, entity: Entity) {
        if let Some(location) = self.location_component.remove(entity) {
            if let Some(layer) = location.layer {
                self.spatial_grid.get_checked_mut(location.coord).clear(layer);
            }
        }
    }
    fn to_serialize(&self) -> SpatialSerialize {
        SpatialSerialize {
            entries: self.location_component.entries().clone(),
            size: self.spatial_grid.size(),
        }
    }
    fn from_serialize(SpatialSerialize { entries, size }: SpatialSerialize) -> Self {
        let location_component = entries.into_component_table();
        let mut spatial_grid: Grid<SpatialCell> = Grid::new_default(size);
        for (entity, location) in location_component.iter() {
            if let Some(layer) = location.layer {
                let cell = spatial_grid.get_checked_mut(location.coord);
                let slot = match layer {
                    Layer::Floor => &mut cell.floor,
                    Layer::Feature => &mut cell.feature,
                    Layer::Character => &mut cell.character,
                };
                assert!(slot.is_none());
                *slot = Some(entity);
            }
        }
        Self {
            location_component,
            spatial_grid,
        }
    }
}

#[derive(Debug)]
pub struct OccupiedBy(pub Entity);

impl SpatialCell {
    fn select_field_mut(&mut self, layer: Layer) -> &mut Option<Entity> {
        match layer {
            Layer::Character => &mut self.character,
            Layer::Feature => &mut self.feature,
            Layer::Floor => &mut self.floor,
        }
    }
    fn insert(&mut self, entity: Entity, layer: Layer) -> Result<(), OccupiedBy> {
        let layer_field = self.select_field_mut(layer);
        if let Some(&occupant) = layer_field.as_ref() {
            Err(OccupiedBy(occupant))
        } else {
            *layer_field = Some(entity);
            Ok(())
        }
    }
    fn clear(&mut self, layer: Layer) -> Option<Entity> {
        self.select_field_mut(layer).take()
    }
}

#[derive(Serialize, Deserialize)]
struct SpatialSerialize {
    entries: ComponentTableEntries<Location>,
    size: Size,
}

impl Serialize for Spatial {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_serialize().serialize(s)
    }
}

impl<'a> Deserialize<'a> for Spatial {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(d).map(Self::from_serialize)
    }
}
