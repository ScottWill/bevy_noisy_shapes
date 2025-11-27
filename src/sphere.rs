use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use fastnoise::{NoiseSampler, Sampler};
use hexasphere::{BaseShape, Subdivided, shapes::{CubeSphere, IcoSphere}};
use std::{f32::consts::{PI, TAU}, fmt::Debug};

#[derive(Clone)]
pub struct NoisySphere {
    pub radius: f32,
    // noise settings
    sampler: NoiseSampler, //todo this should be a builder as well, not the final instance
}

impl Default for NoisySphere {
    fn default() -> Self {
        Self {
            radius: 0.5,
            sampler: NoiseSampler::default(),
        }
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
    pub fn with_sampler(mut self, sampler: NoiseSampler) -> Self {
        self.sampler = sampler;
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

#[derive(Clone, Default)]
pub struct NoisySphereMeshBuilder {
    pub sphere: NoisySphere,
    pub kind: NoisySphereKind,
}

impl NoisySphereMeshBuilder {
    #[inline]
    pub fn new(radius: f32, kind: NoisySphereKind) -> Self {
        Self {
            sphere: NoisySphere { radius, ..Default::default() },
            kind,
        }
    }

    /// Sets the [`SphereKind`] that will be used for building the mesh.
    #[inline]
    pub fn kind(mut self, kind: NoisySphereKind) -> Self {
        self.kind = kind;
        self
    }

    fn cube(&self, subdivisions: u32) -> Mesh {
        mesh(&self.sphere, CubeSphere::new(subdivisions as _, uv_transform), 12)
    }

    fn ico(&self, subdivisions: u32) -> Mesh {
        mesh(&self.sphere, IcoSphere::new(subdivisions as _, uv_transform), 20)
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
fn sample_at(noise: &NoiseSampler, point: Vec3A, radius: f32) -> Vec3A {
    const DEVIATION: f32 = 0.2; // todo: move to final filter
    point * radius * (1.0 + noise.sample3d(point) * DEVIATION)
}

#[inline]
fn uv_transform(point: Vec3A) -> [f32; 2] {
    let inclination = ops::acos(point.y);
    let azimuth = ops::atan2(point.z, point.x);
    let norm_inclination = inclination / PI;
    let norm_azimuth = 0.5 - (azimuth / TAU);
    [norm_azimuth, norm_inclination]
}

fn mesh<S: BaseShape>(sphere: &NoisySphere, base: Subdivided<[f32; 2], S>, faces: usize) -> Mesh {

    let points = base.raw_points()
        .iter()
        .map(|&point| sample_at(&sphere.sampler, point, sphere.radius).into())
        .collect::<Vec<[f32; 3]>>();

    let indices = {
        let mut indices = Vec::with_capacity(base.indices_per_main_triangle() * faces);
        for i in 0..faces {
            base.get_indices(i, &mut indices);
        }
        Indices::U32(indices)
    };

    let uvs = base.raw_data().to_owned();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
        .with_inserted_indices(indices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_computed_normals()
}
