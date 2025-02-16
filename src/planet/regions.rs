use crate::utils::*;

const PHI: f64 = 1.61803398875; // Golden ratio

#[rustfmt::skip]
const ICOS_VERTICES: &[DVec3] = &[
    DVec3::new(-1.0,  PHI,  0.0),
    DVec3::new( 1.0,  PHI,  0.0),
    DVec3::new(-1.0, -PHI,  0.0),
    DVec3::new( 1.0, -PHI,  0.0),
    DVec3::new( 0.0, -1.0,  PHI),
    DVec3::new( 0.0,  1.0,  PHI),
    DVec3::new( 0.0, -1.0, -PHI),
    DVec3::new( 0.0,  1.0, -PHI),
    DVec3::new( PHI,  0.0, -1.0),
    DVec3::new( PHI,  0.0,  1.0),
    DVec3::new(-PHI,  0.0, -1.0),
    DVec3::new(-PHI,  0.0,  1.0),
];

#[rustfmt::skip]
const ICOS_INDICES: &[u16] = &[
    0, 11, 5,  0, 5, 1,  0, 1, 7,  0, 7, 10,  0, 10, 11,
    1, 5, 9,  5, 11, 4,  11, 10, 2,  10, 7, 6,  7, 1, 8,
    3, 9, 4,  3, 4, 2,  3, 2, 6,  3, 6, 8,  3, 8, 9,
    4, 9, 5,  2, 4, 11,  6, 2, 10,  8, 6, 7,  9, 8, 1,
];

fn subdivide(vertices: &mut Vec<DVec3>, indices: &mut Vec<u16>) {
    let mut new_indices = Vec::new();
    let mut midpoint_cache = std::collections::HashMap::new();

    let midpoint = |a: u16,
                    b: u16,
                    vertices: &mut Vec<DVec3>,
                    cache: &mut std::collections::HashMap<(u16, u16), u16>|
     -> u16 {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&mid) = cache.get(&key) {
            return mid;
        }
        let mid_pos = (vertices[a as usize] + vertices[b as usize]) * 0.5;
        let mid_index = vertices.len() as u16;
        vertices.push(mid_pos.normalize());
        cache.insert(key, mid_index);
        mid_index
    };

    for chunk in indices.chunks_exact(3) {
        let m1 = midpoint(chunk[0], chunk[1], vertices, &mut midpoint_cache);
        let m2 = midpoint(chunk[1], chunk[2], vertices, &mut midpoint_cache);
        let m3 = midpoint(chunk[2], chunk[0], vertices, &mut midpoint_cache);

        new_indices.extend_from_slice(&[
            chunk[0], m1, m3, m1, chunk[1], m2, m3, m2, chunk[2], m1, m2, m3,
        ]);
    }
    *indices = new_indices;
}

pub struct Region {
    pub corners: [DVec3; 3],
    pub edges: [u32; 3],
}

impl Region {
    fn new(indices: &[u16], vertices: &[DVec3]) -> Self {
        let a = indices[0] as usize;
        let b = indices[1] as usize;
        let c = indices[2] as usize;
        let ab = ((a.min(b) << 16) | a.max(b)) as u32;
        let bc = ((b.min(c) << 16) | b.max(c)) as u32;
        let ca = ((c.min(a) << 16) | c.max(a)) as u32;
        Region {
            corners: [vertices[a], vertices[b], vertices[c]],
            edges: [ab, bc, ca],
        }
    }

    /// Checks if self borders other (returns true if self and other share an edge)
    pub fn borders(&self, other: &Region) -> bool {
        self.edges.iter().any(|&e| other.edges.contains(&e))
        // self.edges.x == other.edges.x
        //     || self.edges.x == other.edges.y
        //     || self.edges.x == other.edges.z
        //     || self.edges.y == other.edges.y
        //     || self.edges.y == other.edges.z
        //     || self.edges.z == other.edges.z
    }
}

pub fn create_regions(subdivisions: usize) -> Vec<Region> {
    // create vertices by subdividing an icosahedron
    let mut vertices = ICOS_VERTICES.to_owned();
    let mut indices = ICOS_INDICES.to_owned();
    for vertex in &mut vertices {
        *vertex = vertex.normalize();
    }
    for _ in 0..subdivisions {
        subdivide(&mut vertices, &mut indices);
    }
    // create the regions
    let mut regions = vec![];
    for i in 0..indices.len() / 3 {
        regions.push(Region::new(&indices[3 * i..(3 * i + 3)], &vertices));
    }
    regions
}
