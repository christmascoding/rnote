use crate::{compose, render, geometry};

use p2d::bounding_volume::BoundingVolume;
use rand::distributions::Uniform;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::bitmapimage::BitmapImage;
use super::brushstroke::BrushStroke;
use super::markerstroke::MarkerStroke;
use super::shapestroke::ShapeStroke;
use super::vectorimage::VectorImage;

pub trait StrokeBehaviour {
    /// returns the current bounds of this stroke
    fn bounds(&self) -> p2d::bounding_volume::AABB;
    /// sets the bounds of this stroke
    fn set_bounds(&mut self, bounds: p2d::bounding_volume::AABB);
    /// generates the bounds of this stroke
    fn gen_bounds(&self) -> Option<p2d::bounding_volume::AABB> {
        if let Ok(svgs)=  self.gen_svgs(na::vector![0.0, 0.0]) {
            let mut svgs_iter = svgs.iter();
            if let Some(first) = svgs_iter.next() {
                let mut new_bounds = first.bounds;

                svgs_iter.for_each(|svg| {
                    new_bounds.merge(&svg.bounds);
                });
            new_bounds = geometry::aabb_ceil(new_bounds);

                return Some(new_bounds);
            }
        }

        None
    }
    /// translates (as in moves) the type for offset
    fn translate(&mut self, offset: na::Vector2<f64>);
    /// resizes the type to the desired new_bounds
    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB);
    /// generates the svg elements, without the xml header or the svg root.
    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error>;
    /// generates the image for this stroke
    fn gen_image(
        &self,
        zoom: f64,
        renderer: &render::Renderer,
    ) -> Result<render::Image, anyhow::Error> {
        let offset = na::vector![0.0, 0.0];
        let mut svgs = self.gen_svgs(offset)?;

        for svg in svgs.iter_mut() {
            svg.svg_data = compose::wrap_svg(
                svg.svg_data.as_str(),
                Some(self.bounds()),
                Some(self.bounds()),
                true,
                false,
            );
        }

        Ok(renderer.gen_image(zoom, &svgs, self.bounds())?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrokeStyle {
    MarkerStroke(MarkerStroke),
    BrushStroke(BrushStroke),
    ShapeStroke(ShapeStroke),
    VectorImage(VectorImage),
    BitmapImage(BitmapImage),
}

impl StrokeBehaviour for StrokeStyle {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.bounds(),
            Self::BrushStroke(brushstroke) => brushstroke.bounds(),
            Self::ShapeStroke(shapestroke) => shapestroke.bounds(),
            Self::VectorImage(vectorimage) => vectorimage.bounds(),
            Self::BitmapImage(bitmapimage) => bitmapimage.bounds(),
        }
    }

    fn set_bounds(&mut self, bounds: p2d::bounding_volume::AABB) {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.set_bounds(bounds),
            Self::BrushStroke(brushstroke) => brushstroke.set_bounds(bounds),
            Self::ShapeStroke(shapestroke) => shapestroke.set_bounds(bounds),
            Self::VectorImage(vectorimage) => vectorimage.set_bounds(bounds),
            Self::BitmapImage(bitmapimage) => bitmapimage.set_bounds(bounds),
        }
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.translate(offset);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.translate(offset);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.translate(offset);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.translate(offset);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.translate(offset);
            }
        }
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.resize(new_bounds);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.resize(new_bounds);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.resize(new_bounds);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.resize(new_bounds);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.resize(new_bounds);
            }
        }
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.gen_svgs(offset),
            Self::BrushStroke(brushstroke) => brushstroke.gen_svgs(offset),
            Self::ShapeStroke(shapestroke) => shapestroke.gen_svgs(offset),
            Self::VectorImage(vectorimage) => vectorimage.gen_svgs(offset),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_svgs(offset),
        }
    }
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::MarkerStroke(MarkerStroke::default())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputData {
    pos: na::Vector2<f64>,
    pressure: f64,
}

impl Default for InputData {
    fn default() -> Self {
        Self {
            pos: na::vector![0.0, 0.0],
            pressure: Self::PRESSURE_DEFAULT,
        }
    }
}

impl InputData {
    pub const PRESSURE_DEFAULT: f64 = 0.5;

    pub fn new(pos: na::Vector2<f64>, pressure: f64) -> Self {
        let mut inputdata = Self::default();
        inputdata.set_pos(pos);
        inputdata.set_pressure(pressure);

        inputdata
    }

    pub fn pos(&self) -> na::Vector2<f64> {
        self.pos
    }

    pub fn set_pos(&mut self, pos: na::Vector2<f64>) {
        self.pos = pos;
    }

    pub fn pressure(&self) -> f64 {
        self.pressure
    }

    pub fn set_pressure(&mut self, pressure: f64) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }
}

// Represents a single Stroke Element
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Element {
    pub inputdata: InputData,
}

impl Element {
    pub fn new(inputdata: InputData) -> Self {
        Self { inputdata }
    }

    pub fn validation_data(bounds: p2d::bounding_volume::AABB) -> Vec<Self> {
        let mut rng = rand::thread_rng();
        let data_entries_uniform = Uniform::from(0..=20);
        let x_uniform = Uniform::from(bounds.mins[0]..=bounds.maxs[0]);
        let y_uniform = Uniform::from(bounds.mins[1]..=bounds.maxs[1]);
        let pressure_uniform = Uniform::from(0_f64..=1_f64);

        let mut data_entries: Vec<Self> = Vec::new();

        for _i in 0..=data_entries_uniform.sample(&mut rng) {
            data_entries.push(Self::new(InputData::new(
                na::vector![x_uniform.sample(&mut rng), y_uniform.sample(&mut rng)],
                pressure_uniform.sample(&mut rng),
            )));
        }

        data_entries
    }
}
