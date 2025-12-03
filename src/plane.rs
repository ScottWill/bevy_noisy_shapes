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
#[derive(Clone, Debug)]
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
    pub offset: Vec2,
    pub vertex_colors: bool,
}

impl Default for NoisyPlaneMeshBuilder {
    fn default() -> Self {
        Self {
            plane: NoisyPlane3d::default(),
            subdivisions: 0,
            sampler: NoiseSampler::None,
            offset: Vec2::ZERO,
            vertex_colors: false,
        }
    }
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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

    #[inline]
    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    #[inline]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.plane.half_size = Vec2::new(width, height) * 0.5;
        self
    }

    #[inline]
    pub fn square(mut self, width: f32) -> Self {
        self.plane.half_size = Vec2::splat(width) * 0.5;
        self
    }

    #[inline]
    pub fn subdivisions(mut self, subdivisions: u32) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    #[inline]
    pub fn sampler(mut self, sampler: impl Into<NoiseSampler>) -> Self {
        self.sampler = sampler.into();
        self
    }

    #[inline]
    pub fn vertex_colors(mut self, vertex_colors: bool) -> Self {
        self.vertex_colors = vertex_colors;
        self
    }
}

impl MeshBuilder for NoisyPlaneMeshBuilder {
    fn build(&self) -> Mesh {
        let z_vertex_count = self.subdivisions + 2;
        let x_vertex_count = self.subdivisions + 2;
        let num_vertices = (z_vertex_count * x_vertex_count) as usize;
        let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;
        let rotation = Quat::from_rotation_arc(Vec3::Y, *self.plane.normal);
        let size = self.plane.half_size * 2.0;

        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(num_vertices);
        let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;

                let px = (-0.5 + tx) * size.x;
                let pz = (-0.5 + tz) * size.y;
                let py = self.sampler.sample2d([self.offset.x + tx * size.x, self.offset.y + tz * size.y]);
                let pos = rotation * Vec3::new(px, py, pz);
                positions.push(pos);
                uvs.push([tx, tz]);
                if self.vertex_colors {
                    let hue = (py * 360.0) % 360.0;
                    colors.push(Color::hsl(hue, 1.0, 0.5).to_srgba().to_f32_array());
                }
            }
        }

        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);
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

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
            .with_inserted_indices(Indices::U32(indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        if self.vertex_colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }

        mesh.with_computed_normals()
    }
}

impl Meshable for NoisyPlane3d {
    type Output = NoisyPlaneMeshBuilder;

    fn mesh(&self) -> Self::Output {
        NoisyPlaneMeshBuilder {
            plane: *self,
            ..Default::default()
        }
    }
}

impl From<NoisyPlane3d> for Mesh {
    fn from(plane: NoisyPlane3d) -> Self {
        plane.mesh().build()
    }
}