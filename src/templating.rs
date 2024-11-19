use askama::Template;

use crate::{dates::DateInfo, hsl::FormattedStopTime, worker::State};

const MIN_TIME_BUS: usize = 210;
const MIN_TIME_TRAM: usize = 240;
const MIN_TIME_METRO: usize = 150;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Page {
    metro: Vec<FormattedStopTime>,
    tram: Vec<FormattedStopTime>,
    bus: Vec<FormattedStopTime>,
    bus2: Vec<FormattedStopTime>,

    date_info: DateInfo,
}

impl Page {
    pub fn new(state: &State) -> Self {
        let metro: Vec<FormattedStopTime> = state
            .current
            .metro
            .clone()
            .into_iter()
            .filter(|st| st.filter())
            .map(|st| st.calculate_relative_timetables(MIN_TIME_METRO))
            .collect();

        let tram: Vec<FormattedStopTime> = state
            .current
            .tram
            .clone()
            .into_iter()
            .filter(|st| st.filter())
            .map(|st| st.calculate_relative_timetables(MIN_TIME_TRAM))
            .collect();

        let bus: Vec<FormattedStopTime> = state
            .current
            .bus
            .clone()
            .into_iter()
            .filter(|st| st.filter())
            .map(|st| st.calculate_relative_timetables(MIN_TIME_METRO))
            .take(7)
            .collect();

        let bus2: Vec<FormattedStopTime> = state
            .current
            .bus2
            .clone()
            .into_iter()
            .filter(|st| st.filter())
            .map(|st| st.calculate_relative_timetables(MIN_TIME_BUS))
            .take(7)
            .collect();

        Self {
            metro,
            bus,
            bus2,
            tram,
            date_info: DateInfo::calculate(),
        }
    }
}
