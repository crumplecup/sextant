use crate::prelude::*;
use galileo::{Color, MapBuilder};
use galileo::control::{EventPropagation, MouseButton, UserEvent};
use galileo::tile_scheme::TileSchema;
use galileo::layer::feature_layer::FeatureLayer;
use galileo::layer::feature_layer::symbol;
use galileo::render::render_bundle::RenderPrimitive;
use galileo_types::latlon;
use galileo_types::geo::Crs;
use galileo_types::impls::{Contour, Polygon};
use galileo_types::cartesian::CartesianPoint3d;
use galileo_types::geometry::Geom;
use num_traits::AsPrimitive;
use polite::Polite;
use proj::Transform;
use std::sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}};
use tracing::info;

#[derive(Debug, Clone)]
pub struct Viewer;

impl Viewer {
    pub async fn run(cli: &Cli) -> Polite<()> {
        if let Some(path) = &cli.input {
            let parcels = Parcels::load(path)?;
            info!("Parcels read: {}.", parcels.records.len());
        let feature_layer = FeatureLayer::with_lods(
            parcels.records,
            ParcelSymbol {},
            Crs::EPSG3857,
            &vec![8000.0, 1000.0, 1.0],
        );
            let feature_layer = Arc::new(RwLock::new(feature_layer));
            let builder = MapBuilder::new();
            let selected_index = Arc::new(AtomicUsize::new(usize::MAX));
        builder
            .center(latlon!(42.4435, -123.3260))
            .resolution(TileSchema::web(18).lod_resolution(12).unwrap())
            .with_raster_tiles(
                |index| {
                    format!(
                        "https://tile.openstreetmap.org/{}/{}/{}.png",
                        index.z, index.x, index.y
                    )
                },
                TileSchema::web(18),
            )
            .with_layer(feature_layer.clone())
            .with_event_handler(move |ev, map| {
                if let UserEvent::Click(button, event) = ev {
                    if *button == MouseButton::Left {
                        let mut layer = feature_layer.write().unwrap();

                        let Some(position) = map.view().screen_to_map(event.screen_pointer_position)
                        else {
                            return EventPropagation::Stop;
                        };

                        for mut feature_container in
                            layer.get_features_at_mut(&position, map.view().resolution() * 2.0)
                        {
                            info!(
                                "Found {} with bbox {:?}",
                                feature_container.as_ref().owner.id,
                                feature_container.as_ref().bounds
                            );

                            if feature_container.is_hidden() {
                                feature_container.show();
                            } else {
                                feature_container.hide();
                            }
                        }

                        map.redraw();

                        return EventPropagation::Stop;
                    }
                }

                if let UserEvent::PointerMoved(event) = ev {
                    let mut layer = feature_layer.write().unwrap();

                    let mut new_selected = usize::MAX;
                    let Some(position) = map.view().screen_to_map(event.screen_pointer_position) else {
                        return EventPropagation::Stop;
                    };
                    if let Some(feature_container) = layer
                        .get_features_at_mut(&position, map.view().resolution() * 2.0)
                        .next()
                    {
                        let index = feature_container.index();
                        if index == selected_index.load(Ordering::Relaxed) {
                            return EventPropagation::Stop;
                        }

                        feature_container.edit_style().selected = true;
                        new_selected = index;
                    }

                    let selected = selected_index.swap(new_selected, Ordering::Relaxed);
                    if selected != usize::MAX {
                        let feature = layer.features_mut().get_mut(selected).unwrap();
                        feature.edit_style().selected = false;
                    }

                    map.redraw();

                    return EventPropagation::Stop;
                }

                EventPropagation::Propagate
            })
            .build()
            .await
            .run();
        }

        Ok(())
    }
}

pub struct ParcelSymbol;

impl ParcelSymbol {
    pub fn polygon(&self, feature: &Parcel) -> symbol::SimplePolygonSymbol {
        let selected = feature.selected;
        let stroke = {
            if selected {
                Color::BLUE
            } else {
                Color::BLACK
            }
        };
        let mut fill = Color::TRANSPARENT;
        if let Some(name) = &feature.owner.name {
            if name == "CITY OF GRANTS PASS" {
                fill = Color::RED.with_alpha(if selected { 150 } else { 50 });
            }
        }
        symbol::SimplePolygonSymbol::new(fill)
                .with_stroke_color(stroke)
                .with_stroke_width(2.0)
                .with_stroke_offset(-1.0)
    }
}

impl symbol::Symbol<Parcel> for ParcelSymbol {
    fn render<'a, N, P>(
        &self,
        feature: &Parcel,
        geometry: &'a Geom<P>,
        min_resolution: f64,
    ) -> Vec<RenderPrimitive<'a, N, P, Contour<P>, Polygon<P>>>
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N> + Clone,
    {
        self.polygon(feature)
            .render(&(), geometry, min_resolution)
    }
}


