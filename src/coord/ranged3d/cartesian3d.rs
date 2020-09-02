use super::{ProjectionMatrix, ProjectionMatrixBuilder};
use crate::coord::ranged1d::Ranged;
use crate::coord::CoordTranslate;
use plotters_backend::BackendCoord;

use std::ops::Range;

pub struct Cartesian3d<X: Ranged, Y: Ranged, Z: Ranged> {
    pub(crate) logic_x: X,
    pub(crate) logic_y: Y,
    pub(crate) logic_z: Z,
    coord_size: (i32, i32, i32),
    projection: ProjectionMatrix,
}

impl<X: Ranged, Y: Ranged, Z: Ranged> Cartesian3d<X, Y, Z> {
    fn compute_default_size(actual_x: Range<i32>, actual_y: Range<i32>) -> i32 {
        (actual_x.end - actual_x.start).min(actual_y.end - actual_y.start) * 4 / 5
    }
    fn create_projection<F: FnOnce(ProjectionMatrixBuilder) -> ProjectionMatrix>(actual_x: Range<i32>, actual_y: Range<i32>, f: F) -> ProjectionMatrix {
        let default_size = Self::compute_default_size(actual_x.clone(), actual_y.clone());
        let center_3d = (default_size / 2, default_size / 2, default_size / 2);
        let center_2d = (
            (actual_x.end + actual_x.start) / 2,
            (actual_y.end + actual_y.start) / 2,
        );
        let mut pb = ProjectionMatrixBuilder::new();
        pb.set_pivot(center_3d, center_2d);
        f(pb)
    }
    pub fn with_projection<SX: Into<X>, SY: Into<Y>, SZ: Into<Z>, F: FnOnce(ProjectionMatrixBuilder)-> ProjectionMatrix>(
        logic_x: SX,
        logic_y: SY,
        logic_z: SZ,
        (actual_x, actual_y): (Range<i32>, Range<i32>),
        build_projection_matrix: F,
    ) -> Self {
        let default_size = Self::compute_default_size(actual_x.clone(), actual_y.clone());
        Self {
            logic_x: logic_x.into(),
            logic_y: logic_y.into(),
            logic_z: logic_z.into(),
            coord_size: (default_size, default_size, default_size),
            projection: Self::create_projection(actual_x, actual_y, build_projection_matrix),
        }
    }
    pub fn set_projection<F:FnOnce(ProjectionMatrixBuilder) -> ProjectionMatrix>(&mut self, actual_x: Range<i32>, actual_y: Range<i32>, f: F) -> &mut Self {
        self.projection = Self::create_projection(actual_x, actual_y, f);
        self
    }

    pub fn new<SX: Into<X>, SY: Into<Y>, SZ: Into<Z>>(
        logic_x: SX,
        logic_y: SY,
        logic_z: SZ,
        (actual_x, actual_y): (Range<i32>, Range<i32>),
    ) -> Self {
        Self::with_projection(logic_x, logic_y, logic_z, (actual_x, actual_y), |pb| pb.into_matrix())
    }
    pub fn projection(&self) -> &ProjectionMatrix {
        &self.projection
    }

    pub fn map_3d(&self, x: &X::ValueType, y: &Y::ValueType, z: &Z::ValueType) -> (i32, i32, i32) {
        (
            self.logic_x.map(x, (0, self.coord_size.0)),
            self.logic_y.map(y, (0, self.coord_size.1)),
            self.logic_z.map(z, (0, self.coord_size.2)),
        )
    }

    pub fn projected_depth(&self, x: &X::ValueType, y: &Y::ValueType, z: &Z::ValueType) -> i32 {
        self.projection.projected_depth(self.map_3d(x, y, z))
    }
}

impl<X: Ranged, Y: Ranged, Z: Ranged> CoordTranslate for Cartesian3d<X, Y, Z> {
    type From = (X::ValueType, Y::ValueType, Z::ValueType);
    fn translate(&self, coord: &Self::From) -> BackendCoord {
        let pixel_coord_3d = self.map_3d(&coord.0, &coord.1, &coord.2);
        self.projection * pixel_coord_3d
    }
}
