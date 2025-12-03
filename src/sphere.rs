use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use fastnoise::{NoiseSampler, Sampler};
use hexasphere::{BaseShape, Subdivided, shapes::{CubeSphere, IcoSphere}};
use std::{f32::consts::{PI, TAU}, fmt::Debug};

#[derive(Clone)]
pub struct NoisySphere {
    pub radius: f32,
}

impl Default for NoisySphere {
    fn default() -> Self {
        Self { radius: 0.5 }
    }
}

impl NoisySphere {
    pub fn new(radius: f32) -> Self {
        Self { radius, ..Default::default() }
    }
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
#[reflect(Default, Debug, Clone)]
pub enum NoisySphereKind {
    Cubed {
        subdivisions: u32,
    },
    Ico {
        subdivisions: u32,
    },
}

impl Default for NoisySphereKind {
    fn default() -> Self {
        Self::Cubed { subdivisions: 5 }
    }
}

#[derive(Clone)]
pub struct NoisySphereMeshBuilder {
    pub kind: NoisySphereKind,
    pub sampler: NoiseSampler,
    pub sphere: NoisySphere,
    pub offset: Vec3A,
    pub vertex_colors: bool,
}

impl Default for NoisySphereMeshBuilder {
    fn default() -> Self {
        Self {
            kind: NoisySphereKind::default(),
            sampler: NoiseSampler::None,
            sphere: NoisySphere::default(),
            offset: Vec3A::ZERO,
            vertex_colors: false
        }
    }
}

impl NoisySphereMeshBuilder {
    #[inline]
    pub fn new(radius: f32, kind: NoisySphereKind) -> Self {
        Self {
            kind,
            sphere: NoisySphere::new(radius),
            ..Default::default()
        }
    }

    /// Sets the [`SphereKind`] that will be used for building the mesh.
    #[inline]
    pub fn kind(mut self, kind: NoisySphereKind) -> Self {
        self.kind = kind;
        self
    }

    #[inline]
    pub fn sampler(mut self, sampler: impl Into<NoiseSampler>) -> Self {
        self.sampler = sampler.into();
        self
    }

    #[inline]
    pub fn offset(mut self, offset: impl Into<Vec3A>) -> Self {
        self.offset = offset.into();
        self
    }

    #[inline]
    pub fn vertex_colors(mut self, vertex_colors: bool) -> Self {
        self.vertex_colors = vertex_colors;
        self
    }

    fn cube(&self, subdivisions: u32) -> Mesh {
        self.mesh(CubeSphere::new(subdivisions as _, uv_transform), 12)
    }

    fn ico(&self, subdivisions: u32) -> Mesh {
        self.mesh(IcoSphere::new(subdivisions as _, uv_transform), 20)
    }

    fn mesh<S: BaseShape>(
        &self,
        base: Subdivided<[f32; 2], S>,
        faces: usize
    ) -> Mesh {
        let num_vertices = base.raw_points().len();

        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(num_vertices);
        let mut points: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        for &point in base.raw_points() {
            let point = sample_at(point, &self.sampler, self.offset, self.sphere.radius);
            points.push(point.into());
            if self.vertex_colors {
                let hue = (point.length() * 360.0) % 360.0;
                colors.push(Color::hsl(hue, 1.0, 0.5).to_srgba().to_f32_array());
            }
        }

        let indices = {
            let mut indices = Vec::with_capacity(base.indices_per_main_triangle() * faces);
            for i in 0..faces {
                base.get_indices(i, &mut indices);
            }
            Indices::U32(indices)
        };

        let uvs = base.raw_data().to_owned();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
            .with_inserted_indices(indices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        if self.vertex_colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }

        mesh.with_computed_normals()
    }
}

impl MeshBuilder for NoisySphereMeshBuilder {
    fn build(&self) -> Mesh {
        match self.kind {
            NoisySphereKind::Cubed { subdivisions } => self.cube(subdivisions),
            NoisySphereKind::Ico { subdivisions } => self.ico(subdivisions),
        }
    }
}

impl Meshable for NoisySphere {
    type Output = NoisySphereMeshBuilder;

    fn mesh(&self) -> Self::Output {
        NoisySphereMeshBuilder {
            sphere: self.clone(),
            ..Default::default()
        }
    }
}

impl From<NoisySphere> for Mesh {
    fn from(sphere: NoisySphere) -> Self {
        sphere.mesh().build()
    }
}

#[inline]
fn sample_at(point: Vec3A, noise: &NoiseSampler, offset: Vec3A, radius: f32) -> Vec3A {
    point * radius * (1.0 + noise.sample3d(point - offset))
}

#[inline]
fn uv_transform(point: Vec3A) -> [f32; 2] {
    let inclination = ops::acos(point.y);
    let azimuth = ops::atan2(point.z, point.x);
    let norm_inclination = inclination / PI;
    let norm_azimuth = 0.5 - (azimuth / TAU);
    [norm_azimuth, norm_inclination]
}
