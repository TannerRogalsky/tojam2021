#![allow(unused)]

use rapier3d::dynamics::MassProperties;
use rapier3d::geometry::{
    FeatureId, PointProjection, Ray, RayIntersection, Shape, ShapeType, SimdCompositeShape,
    Triangle, TrianglePointLocation, TypedShape, TypedSimdCompositeShape, AABB,
};
use rapier3d::math::{Isometry, Point, Real, Rotation};
use rapier3d::na::RealField;
use rapier3d::parry::bounding_volume::BoundingSphere;
use rapier3d::parry::partitioning::SimdQuadTree;
use rapier3d::parry::query::details::{
    PointCompositeShapeProjWithFeatureBestFirstVisitor,
    PointCompositeShapeProjWithLocationBestFirstVisitor,
    RayCompositeShapeToiAndNormalBestFirstVisitor, RayCompositeShapeToiBestFirstVisitor,
};
use rapier3d::parry::query::visitors::CompositePointContainmentTest;
use rapier3d::parry::query::{PointQuery, PointQueryWithLocation, RayCast};
use std::iter::Map;
use std::slice::ChunksExact;

#[derive(Clone)]
/// A triangle mesh.
pub struct TriMesh {
    quadtree: SimdQuadTree<u32>,
    vertices: Vec<Point<Real>>,
}

impl TriMesh {
    /// Creates a new triangle mesh from a vertex buffer and an index buffer.
    pub fn new(vertices: Vec<Point<Real>>) -> Self {
        debug_assert!(
            vertices.len() > 3,
            "A triangle mesh must contain at least one triangle."
        );

        let data = vertices.chunks_exact(3).enumerate().map(|(i, vertices)| {
            let aabb = Triangle::new(vertices[0], vertices[1], vertices[2]).local_aabb();
            (i as u32, aabb)
        });

        let mut quadtree = SimdQuadTree::new();
        // NOTE: we apply no dilation factor because we won't
        // update this tree dynamically.
        quadtree.clear_and_rebuild(data, 0.0);

        Self { quadtree, vertices }
    }

    /// Compute the axis-aligned bounding box of this triangle mesh.
    pub fn aabb(&self, pos: &Isometry<Real>) -> AABB {
        self.quadtree.root_aabb().transform_by(pos)
    }

    /// Gets the local axis-aligned bounding box of this triangle mesh.
    pub fn local_aabb(&self) -> &AABB {
        self.quadtree.root_aabb()
    }

    /// The acceleration structure used by this triangle-mesh.
    pub fn quadtree(&self) -> &SimdQuadTree<u32> {
        &self.quadtree
    }

    /// The number of triangles forming this mesh.
    pub fn num_triangles(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Does the given feature ID identify a backface of this trimesh?
    pub fn is_backface(&self, feature: FeatureId) -> bool {
        if let FeatureId::Face(i) = feature {
            i >= self.num_triangles() as u32
        } else {
            false
        }
    }

    /// An iterator through all the triangles of this mesh.
    pub fn triangles(&self) -> Map<ChunksExact<'_, Point<Real>>, fn(&[Point<Real>]) -> Triangle> {
        self.vertices
            .chunks_exact(3)
            .map(|vertices| Triangle::new(vertices[0], vertices[1], vertices[2]))
    }

    /// Get the `i`-th triangle of this mesh.
    pub fn triangle(&self, i: u32) -> Triangle {
        let idx = i as usize * 3;
        Triangle::new(
            self.vertices[idx + 0],
            self.vertices[idx + 1],
            self.vertices[idx + 2],
        )
    }

    /// The vertex buffer of this mesh.
    pub fn vertices(&self) -> &[Point<Real>] {
        &self.vertices[..]
    }

    #[inline]
    fn bounding_sphere(&self, pos: &Isometry<Real>) -> BoundingSphere {
        self.local_aabb().bounding_sphere().transform_by(pos)
    }

    /// Computes the local-space bounding sphere of this triangle mesh.
    #[inline]
    fn local_bounding_sphere(&self) -> BoundingSphere {
        self.local_aabb().bounding_sphere()
    }
}

impl SimdCompositeShape for TriMesh {
    fn map_part_at(&self, i: u32, f: &mut dyn FnMut(Option<&Isometry<Real>>, &dyn Shape)) {
        let tri = self.triangle(i);
        f(None, &tri)
    }

    fn quadtree(&self) -> &SimdQuadTree<u32> {
        &self.quadtree
    }
}

impl TypedSimdCompositeShape for TriMesh {
    type PartShape = Triangle;
    type PartId = u32;

    #[inline(always)]
    fn map_typed_part_at(
        &self,
        i: u32,
        mut f: impl FnMut(Option<&Isometry<Real>>, &Self::PartShape),
    ) {
        let tri = self.triangle(i);
        f(None, &tri)
    }

    #[inline(always)]
    fn map_untyped_part_at(&self, i: u32, mut f: impl FnMut(Option<&Isometry<Real>>, &dyn Shape)) {
        let tri = self.triangle(i);
        f(None, &tri)
    }

    fn typed_quadtree(&self) -> &SimdQuadTree<u32> {
        &self.quadtree
    }
}

impl RayCast for TriMesh {
    #[inline]
    fn cast_local_ray(&self, ray: &Ray, max_toi: Real, solid: bool) -> Option<Real> {
        let mut visitor = RayCompositeShapeToiBestFirstVisitor::new(self, ray, max_toi, solid);

        self.quadtree()
            .traverse_best_first(&mut visitor)
            .map(|res| res.1 .1)
    }

    #[inline]
    fn cast_local_ray_and_get_normal(
        &self,
        ray: &Ray,
        max_toi: Real,
        solid: bool,
    ) -> Option<RayIntersection> {
        let mut visitor =
            RayCompositeShapeToiAndNormalBestFirstVisitor::new(self, ray, max_toi, solid);

        self.quadtree()
            .traverse_best_first(&mut visitor)
            .map(|(_, (best, mut res))| {
                // We hit a backface.
                // NOTE: we need this for `TriMesh::is_backface` to work properly.
                if res.feature == FeatureId::Face(1) {
                    res.feature = FeatureId::Face(best + self.vertices().len() as u32)
                } else {
                    res.feature = FeatureId::Face(best);
                }
                res
            })
    }
}

impl PointQuery for TriMesh {
    #[inline]
    fn project_local_point(&self, point: &Point<Real>, solid: bool) -> PointProjection {
        self.project_local_point_and_get_location(point, solid).0
    }

    #[inline]
    fn project_local_point_and_get_feature(
        &self,
        point: &Point<Real>,
    ) -> (PointProjection, FeatureId) {
        let mut visitor =
            PointCompositeShapeProjWithFeatureBestFirstVisitor::new(self, point, false);
        let (proj, (id, _feature)) = self.quadtree().traverse_best_first(&mut visitor).unwrap().1;
        let feature_id = FeatureId::Face(id);
        (proj, feature_id)
    }

    // FIXME: implement distance_to_point too?

    #[inline]
    fn contains_local_point(&self, point: &Point<Real>) -> bool {
        let mut visitor = CompositePointContainmentTest::new(self, point);
        self.quadtree().traverse_depth_first(&mut visitor);
        visitor.found
    }
}

impl PointQueryWithLocation for TriMesh {
    type Location = (u32, TrianglePointLocation);

    #[inline]
    fn project_local_point_and_get_location(
        &self,
        point: &Point<Real>,
        solid: bool,
    ) -> (PointProjection, Self::Location) {
        let mut visitor =
            PointCompositeShapeProjWithLocationBestFirstVisitor::new(self, point, solid);
        self.quadtree().traverse_best_first(&mut visitor).unwrap().1
    }
}

impl Shape for TriMesh {
    fn compute_local_aabb(&self) -> AABB {
        *self.local_aabb()
    }

    fn compute_local_bounding_sphere(&self) -> BoundingSphere {
        self.local_bounding_sphere()
    }

    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }

    fn compute_aabb(&self, position: &Isometry<Real>) -> AABB {
        self.aabb(position)
    }

    fn mass_properties(&self, _density: Real) -> MassProperties {
        MassProperties {
            inv_mass: 0.0,
            inv_principal_inertia_sqrt: rapier3d::na::zero(),
            principal_inertia_local_frame: Rotation::identity(),
            local_com: Point::origin(),
        }
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::TriMesh
    }

    fn as_typed_shape(&self) -> TypedShape {
        TypedShape::Custom(69)
    }

    fn ccd_thickness(&self) -> Real {
        // TODO: in 2D, return the smallest CCD thickness among triangles?
        0.0
    }

    fn ccd_angular_thickness(&self) -> Real {
        // TODO: the value should depend on the angles between
        // adjacent triangles of the trimesh.
        Real::frac_pi_4()
    }

    fn as_composite_shape(&self) -> Option<&dyn SimdCompositeShape> {
        Some(self as &dyn SimdCompositeShape)
    }
}
