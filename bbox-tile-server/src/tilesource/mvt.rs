use crate::tilesource::TileSourceError;
use geozero::{mvt, mvt::Message};

pub struct MvtBuilder {
    tile: mvt::Tile,
    tags: mvt::TagsBuilder<String>,
}

impl MvtBuilder {
    pub fn new() -> Self {
        Self {
            tile: mvt::Tile::default(),
            tags: mvt::TagsBuilder::new(),
        }
    }
    pub fn new_layer(name: &str, tile_size: u32) -> mvt::tile::Layer {
        mvt::tile::Layer {
            version: 2,
            name: String::from(name),
            extent: Some(tile_size),
            ..Default::default()
        }
    }
    pub fn add_feature_attribute(
        &mut self,
        key: &str,
        mvt_value: mvt::tile::Value,
        mvt_feature: &mut mvt::tile::Feature,
    ) -> Result<(), TileSourceError> {
        let (key_idx, val_idx) = self.tags.insert(
            key.to_string(),
            mvt_value
                .try_into()
                .map_err(|_| TileSourceError::MvtEncodeError)?,
        );
        mvt_feature.tags.push(key_idx);
        mvt_feature.tags.push(val_idx);
        Ok(())
    }
    pub fn push_layer(&mut self, mut layer: mvt::tile::Layer) {
        let tags = std::mem::replace(&mut self.tags, mvt::TagsBuilder::new());
        let (keys, values) = tags.into_tags();
        layer.keys = keys;
        layer.values = values.iter().map(|v| (*v).clone().into()).collect();
        self.tile.layers.push(layer);
    }
    pub fn into_blob(self) -> Result<Vec<u8>, TileSourceError> {
        let mut buf = Vec::new();
        self.tile
            .encode(&mut buf)
            .map_err(|_| TileSourceError::MvtEncodeError)?;
        Ok(buf)
    }
}
