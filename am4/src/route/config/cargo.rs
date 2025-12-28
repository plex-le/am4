use crate::route::demand::{CargoDemand, PaxDemand};
use crate::user::{HeavyTraining, LargeTraining};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CargoConfig {
    pub l: u8,
    pub h: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum CargoConfigAlgorithm {
    #[default]
    Auto,
    L,
    H,
}

impl CargoConfig {
    fn calc_l_conf(
        d_pf: PaxDemand,
        capacity: u32,
        l_training: LargeTraining,
        h_training: HeavyTraining,
    ) -> Option<Self> {
        let d_pf_cargo = CargoDemand::from(&d_pf);

        let l_cap = capacity as f32 * 0.7 * (1.0 + l_training.get() as f32 / 100.0);

        if d_pf_cargo.l as f32 > l_cap {
            return Some(CargoConfig { l: 100, h: 0 });
        }

        let l = d_pf_cargo.l as f32 / l_cap;
        let h = 1. - l;
        if (d_pf_cargo.h as f32) < capacity as f32 * h * (1.0 + h_training.get() as f32 / 100.0) {
            None
        } else {
            let lu = (l * 100.0) as u8;

            Some(CargoConfig { l: lu, h: 100 - lu })
        }
    }

    fn calc_h_conf(
        d_pf: PaxDemand,
        capacity: u32,
        l_training: LargeTraining,
        h_training: HeavyTraining,
    ) -> Option<Self> {
        let d_pf_cargo = CargoDemand::from(&d_pf);

        let h_cap = capacity as f32 * (1.0 + h_training.get() as f32 / 100.0);

        if d_pf_cargo.h as f32 > h_cap {
            return Some(CargoConfig { l: 0, h: 100 });
        }

        let h = d_pf_cargo.h as f32 / h_cap;
        let l = 1. - h;
        if (d_pf_cargo.l as f32)
            < capacity as f32 * l * 0.7 * (1.0 + l_training.get() as f32 / 100.0)
        {
            None
        } else {
            let hu = (h * 100.0) as u8;

            Some(CargoConfig { l: 100 - hu, h: hu })
        }
    }

    // Implements a greedy configuration algorithm for cargo aircraft.
    pub fn calculate(
        d_pf: PaxDemand,
        capacity: u32,
        l_training: LargeTraining,
        h_training: HeavyTraining,
        algorithm: CargoConfigAlgorithm,
    ) -> Option<Self> {
        match algorithm {
            CargoConfigAlgorithm::Auto | CargoConfigAlgorithm::L => {
                Self::calc_l_conf(d_pf, capacity, l_training, h_training)
            }
            CargoConfigAlgorithm::H => Self::calc_h_conf(d_pf, capacity, l_training, h_training),
        }
    }
}
