use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;

use crate::models::{OnlineController, OnlineData, OnlinePilot};

#[derive(Clone, Copy, Debug, Default)]
pub struct AirportTrafficCount {
    pub departures: u32,
    pub arrivals: u32,
}

#[derive(Clone, Debug, Default)]
pub struct OnlineCache {
    pub data: OnlineData,
    pilot_by_id: HashMap<String, usize>,
    pilot_by_callsign: HashMap<String, usize>,
    controller_by_id: HashMap<String, usize>,
    airport_traffic: HashMap<String, AirportTrafficCount>,
}

impl OnlineCache {
    pub fn new(data: OnlineData) -> Self {
        let mut cache = Self {
            data,
            ..Self::default()
        };
        cache.rebuild_indexes();
        cache
    }

    pub fn pilot_by_id(&self, id: &str) -> Option<&OnlinePilot> {
        self.pilot_by_id
            .get(id)
            .and_then(|idx| self.data.flights.get(*idx))
    }

    pub fn pilot_by_callsign(&self, callsign: &str) -> Option<&OnlinePilot> {
        self.pilot_by_callsign
            .get(&normalize_key(callsign))
            .and_then(|idx| self.data.flights.get(*idx))
    }

    pub fn controller_by_id(&self, id: &str) -> Option<&OnlineController> {
        self.controller_by_id
            .get(id)
            .and_then(|idx| self.data.controllers.get(*idx))
    }

    pub fn airport_traffic(&self, icao: &str) -> AirportTrafficCount {
        self.airport_traffic
            .get(&normalize_key(icao))
            .copied()
            .unwrap_or_default()
    }

    fn rebuild_indexes(&mut self) {
        for (idx, pilot) in self.data.flights.iter().enumerate() {
            insert_id_key(&mut self.pilot_by_id, &pilot.base.cid, idx);
            insert_id_key(&mut self.pilot_by_id, &pilot.base.session_id, idx);
            insert_id_key(
                &mut self.pilot_by_callsign,
                &normalize_key(&pilot.base.callsign),
                idx,
            );

            if let Some(flight_plan) = &pilot.flight_plan {
                if !flight_plan.departure.is_empty() {
                    self.airport_traffic
                        .entry(normalize_key(&flight_plan.departure))
                        .or_default()
                        .departures += 1;
                }
                if !flight_plan.arrival.is_empty() {
                    self.airport_traffic
                        .entry(normalize_key(&flight_plan.arrival))
                        .or_default()
                        .arrivals += 1;
                }
            }
        }

        for (idx, controller) in self.data.controllers.iter().enumerate() {
            insert_id_key(&mut self.controller_by_id, &controller.base.cid, idx);
            insert_id_key(&mut self.controller_by_id, &controller.base.session_id, idx);
        }
    }
}

// 简单封装一下
pub fn update_cache(cache: &Arc<ArcSwap<OnlineCache>>, data: OnlineData) {
    cache.store(Arc::new(OnlineCache::new(data)));
}

fn insert_id_key(map: &mut HashMap<String, usize>, key: &str, idx: usize) {
    if !key.is_empty() {
        map.entry(key.to_string()).or_insert(idx);
    }
}

fn normalize_key(key: &str) -> String {
    key.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BaseUser, FlightPlan, OnlineController, OnlinePilot};

    #[test]
    fn builds_lookup_indexes_from_online_data() {
        let cache = OnlineCache::new(OnlineData {
            flights: vec![pilot(
                "1001",
                "session-pilot",
                "CCA123",
                Some(flight_plan("ZBAA", "ZSPD")),
            )],
            controllers: vec![controller("2001", "session-controller", "ZBAA_TWR")],
            atis: Vec::new(),
        });

        assert_eq!(
            cache.pilot_by_id("1001").map(|p| p.base.callsign.as_str()),
            Some("CCA123")
        );
        assert_eq!(
            cache
                .pilot_by_id("session-pilot")
                .map(|p| p.base.callsign.as_str()),
            Some("CCA123")
        );
        assert_eq!(
            cache
                .pilot_by_callsign("cca123")
                .map(|p| p.base.cid.as_str()),
            Some("1001")
        );
        assert_eq!(
            cache
                .controller_by_id("session-controller")
                .map(|c| c.base.callsign.as_str()),
            Some("ZBAA_TWR")
        );

        let zbaa = cache.airport_traffic("zbaa");
        let zspd = cache.airport_traffic("ZSPD");
        assert_eq!(zbaa.departures, 1);
        assert_eq!(zbaa.arrivals, 0);
        assert_eq!(zspd.departures, 0);
        assert_eq!(zspd.arrivals, 1);
    }

    fn base_user(cid: &str, session_id: &str, callsign: &str) -> BaseUser {
        BaseUser {
            cid: cid.to_string(),
            name: "Test User".to_string(),
            callsign: callsign.to_string(),
            server: "TEST".to_string(),
            session_id: session_id.to_string(),
            logon_time: "2026-05-15T00:00:00Z".to_string(),
        }
    }

    fn pilot(
        cid: &str,
        session_id: &str,
        callsign: &str,
        flight_plan: Option<FlightPlan>,
    ) -> OnlinePilot {
        OnlinePilot {
            base: base_user(cid, session_id, callsign),
            latitude: 39.9,
            longitude: 116.4,
            altitude: 33000,
            groundspeed: 450,
            transponder: 2200,
            heading: 90,
            bank: 0,
            pitch: 0,
            flight_plan,
        }
    }

    fn controller(cid: &str, session_id: &str, callsign: &str) -> OnlineController {
        OnlineController {
            base: base_user(cid, session_id, callsign),
            frequency: "118.100".to_string(),
            facility: 4,
            rating: 3,
            visual_range: 120,
            text_atis: Vec::new(),
        }
    }

    fn flight_plan(departure: &str, arrival: &str) -> FlightPlan {
        FlightPlan {
            flight_rules: "I".to_string(),
            aircraft: "B738".to_string(),
            departure: departure.to_string(),
            arrival: arrival.to_string(),
            alternate: "ZSSS".to_string(),
            cruise_tas: "450".to_string(),
            altitude: "33000".to_string(),
            deptime: "1200".to_string(),
            enroute_time: "0200".to_string(),
            fuel_time: "0300".to_string(),
            remarks: String::new(),
            route: "DCT".to_string(),
        }
    }
}
