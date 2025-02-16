use super::Region;
use crate::utils::*;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashSet;

fn multi_insert_edge(set: &mut HashSet<u32>, values: &[u32]) {
    for val in values {
        if !set.insert(*val) {
            set.remove(val);
        }
    }
}

fn multi_contains(set: &HashSet<u32>, values: &[u32]) -> bool {
    values.iter().any(|e| set.contains(e))
}

#[derive(Clone, Copy, Default)]
pub enum TectonicPlateClassification {
    #[default]
    Oceanic,
    Continental,
}

#[derive(Default, Clone)]
pub struct TectonicPlate {
    pub classification: TectonicPlateClassification,
    /// the vector of motion for the tectonic plate. Each point on the plate moves
    /// along its coordinate crossed with the vector of motion
    pub motion_axis: DVec3,
    /// contains the indices of the regions inside the tectonic plate
    pub contained_regions: Vec<usize>,
    /// contains the edges forming the border of the tectonic plate
    pub plate_edges: HashSet<u32>,
}

impl TectonicPlate {
    pub fn borders(&self, other: &TectonicPlate) -> bool {
        self.plate_edges
            .iter()
            .any(|e| other.plate_edges.contains(e))
    }

    fn assign_classification(&mut self, rng: &mut impl Rng) {
        if rng.random::<f32>() > 0.6 {
            self.classification = TectonicPlateClassification::Continental;
        } else {
            self.classification = TectonicPlateClassification::Oceanic;
        }
    }
}

pub fn cluster_regions(
    rng: &mut impl Rng,
    regions: &[Region],
    num_plates: usize,
) -> Vec<TectonicPlate> {
    let mut plates = vec![TectonicPlate::default(); num_plates];
    plates
        .iter_mut()
        .for_each(|plate| plate.assign_classification(rng));

    let mut region_indices = (0..regions.len()).collect::<Vec<_>>();
    region_indices.shuffle(rng);

    for i in 0..num_plates {
        let region_index = region_indices.pop().unwrap();
        let region = &regions[region_index];
        plates[i].contained_regions.push(region_index);
        multi_insert_edge(&mut plates[i].plate_edges, &region.edges);
    }

    while let Some(region_index) = region_indices.pop() {
        let region = &regions[region_index];
        if let Some(plate) = plates
            .iter_mut()
            .find(|plate| multi_contains(&plate.plate_edges, &region.edges))
        {
            plate.contained_regions.push(region_index);
            multi_insert_edge(&mut plate.plate_edges, &region.edges);
            plates.shuffle(rng);
        } else {
            region_indices.insert(0, region_index);
        }
    }

    plates
}
