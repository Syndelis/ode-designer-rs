use std::io::Read;

use crate::locale::Locale;

#[derive(Default, Debug)]
pub struct CSVData {
    pub labels: Vec<String>,
    pub lines: Vec<Vec<f64>>,
    pub time: Vec<f64>,
}

#[derive(Default)]
pub struct PlotInfo {
    pub title: String,
    pub xlabel: String,
    pub ylabel: String,
    pub data: CSVData,
}

#[derive(Default)]
pub struct PlotLayout {
    pub rows: u32,
    pub cols: u32,
    pub number_of_tabs: u32,
    pub active_tabs: Vec<u32>,
}

impl CSVData {
    pub fn load_data(reader: impl Read) -> std::io::Result<Self> {
        let mut rdr = csv::Reader::from_reader(reader);

        let mut data = CSVData::default();

        if rdr.has_headers() {
            data.labels = rdr
                .headers()
                .unwrap()
                .iter()
                .map(|v| v.to_string())
                .collect();
        }

        let n_cols = data.labels.len();

        let mut populations: Vec<Vec<f64>> = (0..n_cols).map(|_| Vec::new()).collect();

        for record in rdr.records() {
            record?
                .into_iter()
                .map(|v| v.parse())
                .zip(populations.iter_mut())
                .try_for_each(|(value, population)| {
                    population.push(value?);
                    Result::<(), std::num::ParseFloatError>::Ok(())
                })
                .unwrap();
        }

        data.time = populations.remove(0);
        data.lines = populations;
        data.labels.remove(0);

        Ok(data)
    }

    pub fn population_count(&self) -> usize {
        self.lines[0].len()
    }
}

impl PlotLayout {
    pub fn new(r: u32, c: u32, n_tabs: u32) -> Self {
        Self {
            rows: r,
            cols: c,
            number_of_tabs: n_tabs,
            active_tabs: vec![],
        }
    }
}

impl PlotInfo {
    pub fn new(data: CSVData, locale: &Locale) -> Self {
        Self {
            data,
            title: String::from("TODO!"),
            xlabel: locale.get("default-x-label").to_owned(),
            ylabel: locale.get("default-y-label").to_owned(),
        }
    }

    pub fn update_locale(&mut self, locale: &Locale) {
        self.xlabel = locale.get("default-x-label").to_owned();
        self.ylabel = locale.get("default-y-label").to_owned();
    }
}
