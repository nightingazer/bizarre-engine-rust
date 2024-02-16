use anyhow::Result;
use nalgebra_glm::{Vec2, Vec3};

use crate::{mesh_loader::MeshHandle, vertex::Vertex};

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub id: MeshHandle,
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_box: BoundingBox,
}

pub fn load_meshes_from_obj(
    path: String,
    first_id: usize,
    names: Option<&[String]>,
) -> Result<Vec<Mesh>> {
    let load_options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    };

    let (models, _) = tobj::load_obj(path, &load_options)?;

    let _meshes = Vec::<Mesh>::with_capacity(models.len());

    let id = first_id;

    let meshses = models
        .iter()
        .enumerate()
        .map(|(i, el)| {
            let (vertices, indices, bounding_box) = process_tobj_mesh(&el.mesh)?;
            let name = if let Some(names) = names {
                names.get(i).unwrap_or(&el.name).clone()
            } else {
                el.name.clone()
            };

            let mesh = Mesh {
                id: MeshHandle::new(id + i),
                name,
                vertices,
                indices,
                bounding_box,
            };
            Ok(mesh)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(meshses)
}

fn process_tobj_mesh(mesh: &tobj::Mesh) -> Result<(Vec<Vertex>, Vec<u32>, BoundingBox)> {
    let indices = &mesh.indices;
    let positions = mesh
        .positions
        .chunks(3)
        .map(Vec3::from_column_slice)
        .collect::<Vec<_>>();
    let vertex_count = positions.len();

    let normals = if mesh.normals.is_empty() {
        vec![Vec3::zeros(); vertex_count]
    } else {
        mesh.normals
            .chunks(3)
            .map(Vec3::from_column_slice)
            .collect()
    };

    let texcoords = if mesh.texcoords.is_empty() {
        vec![Vec2::zeros(); vertex_count]
    } else {
        mesh.texcoords
            .chunks(2)
            .map(Vec2::from_column_slice)
            .collect()
    };

    let colors = if mesh.vertex_color.is_empty() {
        vec![Vec3::from([1.0f32; 3]); vertex_count]
    } else {
        mesh.vertex_color
            .chunks(3)
            .map(Vec3::from_column_slice)
            .collect()
    };

    let mut vertices = Vec::with_capacity(vertex_count);
    let mut min = Vec3::from([f32::MAX; 3]);
    let mut max = Vec3::from([f32::MIN; 3]);

    for vert in positions
        .iter()
        .zip(normals.iter())
        .zip(texcoords.iter())
        .zip(colors.iter())
    {
        let (((position, normal), texcoord), color) = vert;
        min.x = min.x.min(position.x);
        min.y = min.y.min(position.y);
        min.z = min.z.min(position.z);
        max.x = max.x.max(position.x);
        max.y = max.y.max(position.y);
        max.z = max.z.max(position.z);

        vertices.push(Vertex {
            position: *position,
            normal: *normal,
            uv: *texcoord,
            color: *color,
        });
    }

    Ok((vertices, indices.to_vec(), BoundingBox { min, max }))
}
