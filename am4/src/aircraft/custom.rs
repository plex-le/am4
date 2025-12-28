use crate::aircraft::EnginePriority;
use crate::aircraft::{Aircraft, AircraftError};
use std::collections::HashSet;
use std::str::FromStr;

/// An [aircraft][Aircraft] that is modified from the base model, for example,
/// by upgrading the engine or changing the game mode.
#[derive(Debug)]
pub struct CustomAircraft {
    pub aircraft: Aircraft, // owned for now
    pub modifiers: Modification,
}

/// A bitset of the specific modification and engine variant
#[derive(Debug, Clone, PartialEq)]
pub struct Modification {
    pub mods: HashSet<Modifier>, // not using Vec to avoid duplicates
    pub engine: EnginePriority,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Modifier {
    Speed,
    Fuel,
    Co2,
    FourX,
    EasyBoost,
}

impl CustomAircraft {
    pub fn from_aircraft_and_modifiers(aircraft: Aircraft, modifiers: Modification) -> Self {
        let mut ac = aircraft;
        let mut cost_mul = 1.0;

        for modifier in modifiers.mods.iter() {
            modifier.apply(&mut ac);
            cost_mul *= modifier.cost_multiplier();
        }
        ac.cost = (ac.cost as f32 * cost_mul).ceil() as u32;

        Self {
            aircraft: ac,
            modifiers,
        }
    }
}

impl Modifier {
    fn apply(&self, aircraft: &mut Aircraft) {
        match self {
            Modifier::Speed => aircraft.speed *= 1.1,
            Modifier::Fuel => aircraft.fuel *= 0.9,
            Modifier::Co2 => aircraft.co2 *= 0.9,
            Modifier::FourX => aircraft.speed *= 4.0,
            Modifier::EasyBoost => aircraft.speed *= 1.5,
        }
    }

    fn cost_multiplier(&self) -> f32 {
        match self {
            Modifier::Speed => 1.07,
            Modifier::Fuel => 1.10,
            Modifier::Co2 => 1.05,
            Modifier::FourX | Modifier::EasyBoost => 1.0,
        }
    }
}

impl Default for Modification {
    fn default() -> Self {
        Modification {
            mods: HashSet::new(),
            engine: EnginePriority(0),
        }
    }
}

impl FromStr for Modification {
    type Err = AircraftError;

    /// Parse modifiers between square brackets: engine priority (digits) and modifier letters.
    /// Examples: "1", "sfc", "1sfc", "10", "2sfcx"
    fn from_str(s: &str) -> Result<Modification, Self::Err> {
        let mut modifi = Modification::default();
        let s_lower = s.to_lowercase();

        let mut digit_start = 0;
        let mut digit_end = 0;
        let mut found_digit = false;

        for (i, c) in s_lower.chars().enumerate() {
            if c.is_ascii_digit() {
                if !found_digit {
                    digit_start = i;
                    found_digit = true;
                }
                digit_end = i + 1;
            }
        }

        if found_digit {
            let digit_str = &s_lower[digit_start..digit_end];
            modifi.engine = EnginePriority::from_str(digit_str)?;
        }

        for c in s_lower.chars() {
            match c {
                's' => modifi.mods.insert(Modifier::Speed),
                'f' => modifi.mods.insert(Modifier::Fuel),
                'c' => modifi.mods.insert(Modifier::Co2),
                'x' => modifi.mods.insert(Modifier::FourX),
                'e' => modifi.mods.insert(Modifier::EasyBoost),
                ' ' | ',' => continue,
                _ => true,
            };
        }

        Ok(modifi)
    }
}
