use crate::datasource::{mvt::MvtBuilder, TileSourceError};
use crate::service::QueryExtent;
use geozero::{mvt, ToMvt};
use tile_grid::Xyz;

/// Diagnostics tile layer.
pub fn diagnostics_layer(
    tile: &Xyz,
    extent_info: &QueryExtent,
) -> Result<mvt::tile::Layer, TileSourceError> {
    let extent = &extent_info.extent;
    let mut mvt = MvtBuilder::new();
    const SIZE: u32 = 4096;
    const SIZE_F: f64 = 4096.0;
    let mut mvt_layer = MvtBuilder::new_layer("tile", SIZE);
    let geom: geo_types::Geometry<f64> = geo_types::Polygon::new(
        geo_types::LineString::from(vec![
            (0., 0.),
            (0., SIZE_F),
            (SIZE_F, SIZE_F),
            (SIZE_F, 0.),
            (0., 0.),
        ]),
        vec![],
    )
    .into();
    let mut feat = geom.to_mvt_unscaled()?;
    mvt.add_feature_attribute("x", mvt::TileValue::Uint(tile.x).into(), &mut feat)?;
    mvt.add_feature_attribute("y", mvt::TileValue::Uint(tile.y).into(), &mut feat)?;
    mvt.add_feature_attribute("z", mvt::TileValue::Uint(tile.z as u64).into(), &mut feat)?;
    mvt.add_feature_attribute("top", mvt::TileValue::Double(extent.top).into(), &mut feat)?;
    mvt.add_feature_attribute(
        "left",
        mvt::TileValue::Double(extent.left).into(),
        &mut feat,
    )?;
    mvt.add_feature_attribute(
        "bottom",
        mvt::TileValue::Double(extent.bottom).into(),
        &mut feat,
    )?;
    mvt.add_feature_attribute(
        "right",
        mvt::TileValue::Double(extent.right).into(),
        &mut feat,
    )?;
    mvt_layer.features.push(feat);
    Ok(mvt_layer)
}
