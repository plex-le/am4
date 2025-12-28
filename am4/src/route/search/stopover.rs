use crate::{
    aircraft::Aircraft,
    airport::Airport,
    route::{db::DistanceMatrix, Distance},
    user::GameMode,
};

use super::AbstractRoute;

#[derive(Debug, Clone)]
pub struct Stopover<'a>(pub &'a Airport);

impl<'a> Stopover<'a> {
    fn is_candidate_valid(
        candidate: &Airport,
        origin: &Airport,
        destination: &Airport,
        rwy_req: u16,
    ) -> bool {
        if candidate.id == origin.id || candidate.id == destination.id {
            return false;
        }
        if candidate.rwy < rwy_req {
            return false;
        }
        true
    }

    /// given an origin O, destination D, and range R, find the best intermediate stopover S,
    /// such that distance(O, S) + distance(S, D) is minimised, subject to distance < R.
    pub fn find_by_efficiency(
        airports: &'a [Airport],
        distances: &DistanceMatrix,
        origin: &Airport,
        destination: &Airport,
        aircraft: &Aircraft,
        game_mode: GameMode,
    ) -> Option<(Self, Distance)> {
        let mut best_stopover: Option<Self> = None;
        let mut best_dist_total = Distance::MAX;

        let rwy_req = if game_mode == GameMode::Realism {
            aircraft.rwy
        } else {
            0
        };

        for candidate in airports.iter() {
            if !Self::is_candidate_valid(candidate, origin, destination, rwy_req) {
                continue;
            }

            let Ok(inbound) = AbstractRoute::new(distances, origin, candidate) else {
                continue;
            };
            if !inbound.distance_valid(aircraft) {
                continue;
            }

            let Ok(outbound) = AbstractRoute::new(distances, destination, candidate) else {
                continue;
            };
            if !outbound.distance_valid(aircraft) {
                continue;
            }

            let dist_total = inbound.direct_distance + outbound.direct_distance;
            if dist_total < best_dist_total {
                best_stopover = Some(Self(candidate));
                best_dist_total = dist_total;
            }
        }
        best_stopover.map(|s| (s, best_dist_total))
    }

    /// Find a stopover that inflates the total distance to be as close to `target_distance` as possible
    /// without exceeding it, for contribution maximization.
    pub fn find_by_target_distance_lt(
        airports: &'a [Airport],
        distances: &DistanceMatrix,
        origin: &Airport,
        destination: &Airport,
        aircraft: &Aircraft,
        game_mode: GameMode,
        target_distance: Distance,
    ) -> Option<(Self, Distance)> {
        let mut best_stopover: Option<Self> = None;
        let mut best_dist_total = Distance::MIN;

        let rwy_req = if game_mode == GameMode::Realism {
            aircraft.rwy
        } else {
            0
        };

        for candidate in airports.iter() {
            if !Self::is_candidate_valid(candidate, origin, destination, rwy_req) {
                continue;
            }

            let Ok(inbound) = AbstractRoute::new(distances, origin, candidate) else {
                continue;
            };
            if !inbound.distance_valid(aircraft) {
                continue;
            }

            let Ok(outbound) = AbstractRoute::new(distances, destination, candidate) else {
                continue;
            };
            if !outbound.distance_valid(aircraft) {
                continue;
            }

            let dist_total = inbound.direct_distance + outbound.direct_distance;
            if dist_total <= target_distance && dist_total > best_dist_total {
                best_stopover = Some(Self(candidate));
                best_dist_total = dist_total;
            }
        }
        best_stopover.map(|s| (s, best_dist_total))
    }
}
