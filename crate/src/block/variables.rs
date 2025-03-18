use super::{NewFormatHeader, OldFormatHeader};

#[derive(Clone, Debug)]
struct Variable {
    symbol: String,
    label: String,
    units: String,
    data: Vec<f64>,
    is_dependent: bool,
}

#[derive(Clone, Debug)]
pub(super) struct FromTo {
    pub(super) from: f64,
    pub(super) to: f64,
    pub(super) length: usize,
}

impl FromTo {
    pub(super) fn values(&self) -> Vec<f64> {
        let step = (self.to - self.from) / ((self.length - 1) as f64);

        (0..self.length)
            .map(|i| self.from + i as f64 * step)
            .collect()
    }
}

#[derive(Clone, Debug)]
pub(super) struct MeasurementXYVariables {
    x: Variable,
    y: Variable,
}

impl MeasurementXYVariables {
    pub(super) fn new(x: Vec<f64>, y: Vec<f64>, header: &OldFormatHeader) -> Self {
        let (ordered_x, ordered_y) = ensure_increasing(x, y);

        // let rx = regex::Regex::new(r"/(?<label>.*?) ?[([](?<units>.*)[)\]]/").unwrap();

        MeasurementXYVariables {
            x: Variable {
                symbol: "x".to_owned(),
                label: header.xyz_labels.clone(),
                units: format!("{:?}", header.x_unit_type).to_string(),
                data: ordered_x,
                is_dependent: false,
            },
            y: Variable {
                symbol: "x".to_owned(),
                label: header.xyz_labels.clone(),
                units: format!("{:?}", header.y_unit_type).to_string(),
                data: ordered_y,
                is_dependent: true,
            },
        }
    }

    pub(super) fn new_new(x: Vec<f64>, y: Vec<f64>, header: &NewFormatHeader) -> Self {
        let (ordered_x, ordered_y) = ensure_increasing(x, y);

        // let rx = regex::Regex::new(r"/(?<label>.*?) ?[([](?<units>.*)[)\]]/").unwrap();

        MeasurementXYVariables {
            x: Variable {
                symbol: "x".to_owned(),
                label: header.xyz_labels.clone(),
                units: format!("{:?}", header.x_unit_type).to_string(),
                data: ordered_x,
                is_dependent: false,
            },
            y: Variable {
                symbol: "x".to_owned(),
                label: header.xyz_labels.clone(),
                units: format!("{:?}", header.y_unit_type).to_string(),
                data: ordered_y,
                is_dependent: true,
            },
        }
    }
}

fn ensure_increasing(mut x: Vec<f64>, mut y: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    if x[0] > x[x.len() - 1] {
        x.reverse();
        y.reverse();
    }
    (x, y)
}
