use askama::Template;

use crate::{dates::DateInfo, hsl::FormattedStopTime, worker::State};

const MIN_TIME: usize = 120;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Page {
    metro: Vec<FormattedStopTime>,
    tram: Vec<FormattedStopTime>,
    bus: Vec<FormattedStopTime>,

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
            .map(|st| st.calculate_relative_timetables(MIN_TIME))
            .collect();

        let tram: Vec<FormattedStopTime> = state
            .current
            .tram
            .clone()
            .into_iter()
            .filter(|st| st.filter())
            .map(|st| st.calculate_relative_timetables(MIN_TIME))
            .collect();

        let bus: Vec<FormattedStopTime> = state
            .current
            .bus
            .clone()
            .into_iter()
            .filter(|st| st.filter())
            .map(|st| st.calculate_relative_timetables(MIN_TIME))
            .collect();

        Self {
            metro,
            bus,
            tram,
            date_info: DateInfo::calculate(),
        }
    }
}
