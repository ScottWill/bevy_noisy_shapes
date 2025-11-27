use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use fastnoise::{NoiseSampler, Sampler};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct NoisyPlane3d {
    /// The normal of the plane. The plane will be placed perpendicular to this direction
    pub normal: Dir3,
    /// Half of the width and height of the plane
    pub half_size: Vec2,
}

impl Default for NoisyPlane3d {
    /// Returns the default [`Plane3d`] with a normal pointing in the `+Y` direction, width and height of `1.0`.
    fn default() -> Self {
        Self {
            normal: Dir3::Y,
            half_size: Vec2::splat(0.5),
        }
    }
}

impl Primitive3d for NoisyPlane3d {}

/// A builder used for creating a [`Mesh`] with a [`Plane3d`] shape.
#[derive(Clone, Debug, Default)]
pub struct NoisyPlaneMeshBuilder {
    /// The [`Plane3d`] shape.
    pub plane: NoisyPlane3d,
    /// The number of subdivisions in the mesh.
    ///
    /// 0 - is the original plane geometry, the 4 points in the XZ plane.
    ///
    /// 1 - is split by 1 line in the middle of the plane on both the X axis and the Z axis, resulting in a plane with 4 quads / 8 triangles.
    ///
    /// 2 - is a plane split by 2 lines on both the X and Z axes, subdividing the plane into 3 equal sections along each axis, resulting in a plane with 9 quads / 18 triangles.
    ///
    /// and so on...
    pub subdivisions: u32,

    pub sampler: NoiseSampler,
}

impl NoisyPlaneMeshBuilder {
    /// Creates a new [`PlaneMeshBuilder`] from a given normal and size.
    #[inline]
    pub fn new(normal: Dir3, size: Vec2) -> Self {
        Self {
            plane: NoisyPlane3d {
                normal,
                half_size: size / 2.0,
            },
            subdivisions: 0,
            sampler: NoiseSampler::None,
        }
    }

    /// Creates a new [`PlaneMeshBuilder`] from the given size, with the normal pointing upwards.
    #[inline]
    pub fn from_size(size: Vec2) -> Self {
        Self {
            plane: NoisyPlane3d {
                half_size: size / 2.0,
                ..Default::default()
            },
            subdivisions: 0,
            sampler: NoiseSampler::None,
        }
    }

    /// Creates a new [`PlaneMeshBuilder`] from the given length, with the normal pointing upwards,
    /// and the resulting [`PlaneMeshBuilder`] being a square.
    #[inline]
    pub fn from_length(length: f32) -> Self {
        Self {
            plane: NoisyPlane3d {
                half_size: Vec2::splat(length) / 2.0,
                ..Default::default()
            },
            subdivisions: 0,
            sampler: NoiseSampler::None,
        }
    }

    /// Sets the normal of the plane, aka the direction the plane is facing.
    #[inline]
    #[doc(alias = "facing")]
    pub fn normal(mut self, normal: Dir3) -> Self {
        self.plane = NoisyPlane3d {
            normal,
            ..self.plane
        };
        self
    }

    /// Sets the size of the plane mesh.
    #[inline]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.plane.half_size = Vec2::new(width, height) / 2.0;
        self
    }

    /// Sets the subdivisions of the plane mesh.
    ///
    /// 0 - is the original plane geometry, the 4 points in the XZ plane.
    ///
    /// 1 - is split by 1 line in the middle of the plane on both the X axis and the Z axis,
    ///     resulting in a plane with 4 quads / 8 triangles.
    ///
    /// 2 - is a plane split by 2 lines on both the X and Z axes, subdividing the plane into 3
    ///     equal sections along each axis, resulting in a plane with 9 quads / 18 triangles.
    #[inline]
    pub fn subdivisions(mut self, subdivisions: u32) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    #[inline]
    pub fn sampler(mut self, sampler: NoiseSampler) -> Self {
        self.sampler = sampler;
        self
    }
}

impl MeshBuilder for NoisyPlaneMeshBuilder {
    fn build(&self) -> Mesh {
        let z_vertex_count = self.subdivisions + 2;
        let x_vertex_count = self.subdivisions + 2;
        let num_vertices = (z_vertex_count * x_vertex_count) as usize;
        let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

        let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        let rotation = Quat::from_rotation_arc(Vec3::Y, *self.plane.normal);
        let size = self.plane.half_size * 2.0;

        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;

                let px = (-0.5 + tx) * size.x;
                let pz = (-0.5 + tz) * size.y;
                let py = self.sampler.sample2d([px, pz]);
                let pos = rotation * Vec3::new(px, py, pz);
                positions.push(pos);
                normals.push(self.plane.normal.to_array());
                uvs.push([tx, tz]);
            }
        }

        for z in 0..z_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = z * x_vertex_count + x;
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl Meshable for NoisyPlane3d {
    type Output = NoisyPlaneMeshBuilder;

    fn mesh(&self) -> Self::Output {
        NoisyPlaneMeshBuilder {
            plane: *self,
            subdivisions: 0,
            sampler: NoiseSampler::None,
        }
    }
}

impl From<NoisyPlane3d> for Mesh {
    fn from(plane: NoisyPlane3d) -> Self {
        plane.mesh().build()
    }
}
