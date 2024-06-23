use std::collections::HashMap;

use super::{Vec2f, Vec3f};



/// OBJ Parsing function
/// * `lines` - line iterator
/// * Returns opitonal vertex-index array
pub fn parse_obj<'t>(lines: impl Iterator<Item = &'t str>) -> Option<(Vec<crate::animation::render::Vertex>, Vec<u32>)> {
    let mut positions = vec![Vec3f::new(0.0, 0.0, 0.0)];
    let mut tex_coords = vec![Vec2f::new(0.0, 0.0)];
    let mut normals = vec![Vec3f::new(0.0, 0.0, 0.0)];

    let mut index_table = HashMap::<(u32, u32, u32), u32>::new();
    let mut vertices = Vec::<crate::animation::render::Vertex>::new();
    let mut indices = Vec::<u32>::new();

    for line in lines {
        let mut segments = line.split(' ').filter_map(|l| if l.is_empty() { None } else { Some(l.trim()) });

        let ty = match segments.next() {
            Some(v) => v,
            None => continue
        };

        match ty {
            "v" => positions.push(Vec3f {
                x: segments.next()?.parse::<f32>().ok()?,
                y: segments.next()?.parse::<f32>().ok()?,
                z: segments.next()?.parse::<f32>().ok()?,
            }),
            "vt" => tex_coords.push(Vec2f {
                x: segments.next()?.parse::<f32>().ok()?,
                y: segments.next()?.parse::<f32>().ok()?,
            }),
            "vn" => normals.push(Vec3f {
                x: segments.next()?.parse::<f32>().ok()?,
                y: segments.next()?.parse::<f32>().ok()?,
                z: segments.next()?.parse::<f32>().ok()?,
            }),
            "f" => {
                let mut idx = segments
                    .map(|v| -> Option<u32> {
                        let mut sp = v.split('/');

                        let tup = (
                            sp.next()?.parse::<u32>().ok()?,
                            sp.next()?.parse::<u32>().unwrap_or(0),
                            sp.next()?.parse::<u32>().ok()?,
                        );

                        if let Some(entry) = index_table.get(&tup) {
                            Some(*entry)
                        } else {
                            let i = vertices.len() as u32;

                            vertices.push(crate::animation::render::Vertex {
                                position: *positions.get(tup.0 as usize)?,
                                tex_coord: *tex_coords.get(tup.1 as usize)?,
                                normal: *normals.get(tup.2 as usize)?,
                            });

                            index_table.insert(tup, i);

                            Some(i)
                        }
                    })
                    .flatten();

                let base = idx.next()?;
                let mut current = idx.next()?;

                'face: loop {
                    let new = match idx.next() {
                        Some(v) => v,
                        None => break 'face,
                    };

                    indices.push(base);
                    indices.push(current);
                    indices.push(new);

                    current = new;
                }
            },
            _ => continue,
        }
    }

    if indices.is_empty() {
        None
    } else {
        Some((vertices, indices))
    }
} // fn parse_obj

// file obj.rs
