use anyhow::Result;

use crate::vertex::VertexData;

pub struct Mesh {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn from_obj(path: String) -> Result<Self> {
        let (models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,

                ..Default::default()
            },
        )?;

        let mesh = &models[0].mesh;
        let mut vertices: Vec<VertexData> = Vec::with_capacity(mesh.positions.len() / 3);
        for i in (0..(mesh.positions.len())).step_by(3) {
            let vertex = VertexData {
                position: [
                    mesh.positions[i],
                    mesh.positions[i + 1],
                    mesh.positions[i + 2],
                ]
                .into(),
                normal: [mesh.normals[i], mesh.normals[i + 1], mesh.normals[i + 2]].into(),
                color: [1.0, 1.0, 1.0].into(),
            };

            vertices.push(vertex);
        }

        Ok(Self {
            vertices,
            indices: mesh.indices.clone(),
        })
    }
}
