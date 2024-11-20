use std::{error::Error, sync::Arc};

use chrono::Timelike;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::worker::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HslResponse {
    pub data: HslResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HslResponseData {
    pub stations: Vec<Station>,
    pub bus: Vec<Stop>,
    pub bus2: Vec<Stop>,
    pub tram: Vec<Stop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub name: String,
    pub stops: Vec<Stop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub gtfs_id: String,
    pub name: String,
    pub code: String,
    pub routes: Vec<Route>,
    pub stoptimes_without_patterns: Vec<StopTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub long_name: String,
    pub short_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopTime {
    pub scheduled_arrival: i32,
    pub realtime_arrival: i32,
    pub arrival_delay: i32,
    pub scheduled_departure: i32,
    pub realtime_departure: i32,
    pub departure_delay: i32,
    pub service_day: i32,
    pub headsign: String,
}

#[derive(Debug, Clone)]
pub struct FormattedStopTime {
    pub _inner: StopTime,
    pub scheduled_arrival: String,
    pub realtime_arrival: String,
    pub arrival_delay: String,
    pub scheduled_departure: String,
    pub realtime_departure: String,

    pub relative_departure: String,
    pub relative_arrival: String,

    pub full_name: String,
    pub short_name: String,

    pub in_time_window: bool,
}

impl FormattedStopTime {
    pub fn time_to_string(raw: i32) -> String {
        let neg = if raw < 0 { -1 } else { 1 };
        let raw = raw.abs();

        let _seconds = raw as f32;
        let _minutes = _seconds / 60.;
        let hours = (_minutes / 60.).floor();
        let __minutes = _minutes % 60.;
        let minutes = __minutes.floor();
        let seconds = ((__minutes - minutes) * 60.).round();

        let hours = hours as i32;
        let minutes = minutes as i32;
        let seconds = seconds as i32;

        let timeformat = match (hours != 0, minutes != 0, seconds != 0) {
            (true, true, true)
            | (true, true, false)
            | (true, false, false)
            | (true, false, true) => TimeFormat::Hours(neg * hours, minutes, seconds),
            (false, true, true) | (false, true, false) => {
                TimeFormat::Minutes(neg * minutes, seconds)
            }
            (false, false, true) => TimeFormat::Seconds(neg * seconds),
            (false, false, false) => TimeFormat::Now,
        };

        timeformat.into()
    }

    pub fn calculate_relative_timetables(mut self, min: usize) -> Self {
        let now = chrono::offset::Local::now();
        let seconds = ((now.hour() * 60 * 60) + (now.minute() * 60) + now.second()) as i32;

        let rel_arrival = Self::time_to_string(self._inner.realtime_arrival - seconds);
        let rel_departure = Self::time_to_string(self._inner.realtime_departure - seconds);

        self.relative_arrival = rel_arrival;
        self.relative_departure = rel_departure;

        self.in_time_window = (self._inner.realtime_arrival - seconds) > min as i32;

        self
    }

    pub fn filter(&self) -> bool {
        !self.full_name.contains("Otaniemi")
    }
}

impl TryFrom<(&StopTime, &Vec<Stop>)> for FormattedStopTime {
    type Error = Box<dyn Error>;

    fn try_from(value: (&StopTime, &Vec<Stop>)) -> Result<Self, Self::Error> {
        let (stop_time, stops) = value;

        let _inner = stop_time.clone();

        let schedules_arrival = Self::time_to_string(stop_time.scheduled_arrival);
        let realtime_arrival = Self::time_to_string(stop_time.realtime_arrival);
        let arrival_delay = Self::time_to_string(stop_time.arrival_delay);
        let scheduled_departure = Self::time_to_string(stop_time.scheduled_departure);
        let realtime_departure = Self::time_to_string(stop_time.realtime_departure);
        let headsign = stop_time.headsign.clone();

        let mut destination = stop_time
            .headsign
            .split(" ")
            .map(|l| l.split("-"))
            .flatten()
            .nth(0)
            .ok_or(headsign.clone())?;
        let routes = stops.iter().fold(vec![], |mut a, v| {
            let mut routes = v.routes.clone();
            a.append(&mut routes);
            a
        });

        if destination == "Westendinas." {
            destination = "Westendinasema"; // Fuck it
        }

        let route = routes
            .iter()
            .find_map(|route| {
                if route.long_name.contains(destination) {
                    Some(route)
                } else {
                    None
                }
            })
            .ok_or(headsign.clone())?;

        let full_name = format!("{} {}", route.short_name, headsign);
        let short_name = route.short_name.clone();

        let now = chrono::offset::Local::now();
        let seconds = ((now.hour() * 60 * 60) + (now.minute() * 60) + now.second()) as i32;

        let rel_arrival = Self::time_to_string(stop_time.realtime_arrival - seconds);
        let rel_departure = Self::time_to_string(stop_time.realtime_departure - seconds);

        Ok(Self {
            _inner,
            scheduled_arrival: schedules_arrival,
            realtime_arrival,
            arrival_delay,
            scheduled_departure,
            realtime_departure,
            full_name,
            short_name,
            relative_departure: rel_departure,
            relative_arrival: rel_arrival,
            in_time_window: true,
        })
    }
}

pub enum TimeFormat {
    Now,
    Seconds(i32),
    Minutes(i32, i32),
    Hours(i32, i32, i32),
}

#[derive(Debug, Clone, Default)]
pub struct HslResult {
    pub metro: Vec<FormattedStopTime>,
    pub tram: Vec<FormattedStopTime>,
    pub bus: Vec<FormattedStopTime>,
    pub bus2: Vec<FormattedStopTime>,
}

impl Into<String> for TimeFormat {
    fn into(self) -> String {
        match self {
            TimeFormat::Now => format!("0s"),
            TimeFormat::Seconds(s) => format!("{}s", s),
            TimeFormat::Minutes(m, s) => format!("{}min {}s", m, s),
            TimeFormat::Hours(h, m, _s) => format!("{:02}:{:02}", h, m),
        }
    }
}

#[derive(Clone)]
pub struct ApiClient {
    _inner: Client,
    _api_key: String,
}

pub async fn create_client(api_key: String) -> Result<ApiClient, Box<dyn Error>> {
    let mut headers = HeaderMap::default();
    headers.insert(CONTENT_TYPE, "application/json".parse()?);

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;

    Ok(ApiClient {
        _inner: client,
        _api_key: api_key,
    })
}

pub async fn fetch_data(client: ApiClient) -> Result<HslResult, Box<dyn Error>> {
    let api_key = client._api_key;
    let url = format!("https://api.digitransit.fi/routing/v1/routers/hsl/index/graphql?digitransit-subscription-key={}", api_key);

    let graphql = r#"
        {
        "query": "{\tstations(ids: \"HSL:2000102\") { name stops { gtfsId name code platformCode, routes { longName, shortName }, stoptimesWithoutPatterns { scheduledArrival realtimeArrival arrivalDelay scheduledDeparture realtimeDeparture departureDelay realtime serviceDay headsign } } } bus: stops(ids: [\"HSL:2222234\", \"HSL:2222212\"]) { gtfsId name code platformCode, routes { longName, shortName }, stoptimesWithoutPatterns { scheduledArrival realtimeArrival arrivalDelay scheduledDeparture realtimeDeparture departureDelay realtime serviceDay headsign } }, bus2: stops(ids: [\"HSL:2213270\", \"HSL:2213271\"]) { gtfsId name code platformCode, routes { longName, shortName }, stoptimesWithoutPatterns { scheduledArrival realtimeArrival arrivalDelay scheduledDeparture realtimeDeparture departureDelay realtime serviceDay headsign } }, tram: stops(ids: [\"HSL:2222405\", \"HSL:2222406\"]) { gtfsId name code platformCode, routes { longName, shortName }, stoptimesWithoutPatterns { scheduledArrival realtimeArrival arrivalDelay scheduledDeparture realtimeDeparture departureDelay realtime serviceDay headsign } }}"
        }
    "#;

    let mut headers = HeaderMap::default();
    headers.insert(CONTENT_LENGTH, HeaderValue::from(graphql.bytes().len()));

    let resp = client
        ._inner
        .post(url)
        .headers(headers)
        .body(graphql)
        .send()
        .await?;

    let body = resp.json::<HslResponse>().await?;

    let mut metro = vec![];
    let mut bus = vec![];
    let mut bus2 = vec![];
    let mut tram = vec![];

    if let Some(station) = body.data.stations.iter().nth(0) {
        let stops = &station.stops;
        for stop in stops.iter() {
            for st in stop.stoptimes_without_patterns.iter() {
                let fst = FormattedStopTime::try_from((st, stops))?;
                metro.push(fst);
            }
        }
    }

    let stops = &body.data.bus;
    for stop in stops.iter() {
        for st in stop.stoptimes_without_patterns.iter() {
            let fst = FormattedStopTime::try_from((st, stops))?;
            bus.push(fst);
        }
    }

    let stops = &body.data.bus2;
    for stop in stops.iter() {
        for st in stop.stoptimes_without_patterns.iter() {
            let fst = FormattedStopTime::try_from((st, stops))?;
            bus2.push(fst);
        }
    }

    let stops = &body.data.tram;
    for stop in stops.iter() {
        for st in stop.stoptimes_without_patterns.iter() {
            let fst = FormattedStopTime::try_from((st, stops))?;
            tram.push(fst);
        }
    }

    metro.sort_by(|a, b| a._inner.realtime_arrival.cmp(&b._inner.realtime_arrival));
    bus.sort_by(|a, b| a._inner.realtime_arrival.cmp(&b._inner.realtime_arrival));
    bus2.sort_by(|a, b| a._inner.realtime_arrival.cmp(&b._inner.realtime_arrival));
    tram.sort_by(|a, b| a._inner.realtime_arrival.cmp(&b._inner.realtime_arrival));

    Ok(HslResult {
        metro,
        bus,
        bus2,
        tram,
    })
}

pub async fn update_state(
    state: Arc<RwLock<State>>,
    client: ApiClient,
) -> Result<(), Box<dyn Error>> {
    let result = fetch_data(client).await?;

    let mut w_lock = state.write().await;
    w_lock.current = result;
    drop(w_lock);

    Ok(())
}
