mod indicators;
use std::fmt::Display;

pub use indicators::KeyIndicator;
use indicators::*;

#[derive(Clone, Debug)]
pub enum Indicator {
    I1D(Indicator1D),
    I2D(Indicator2D),
    I3D(Indicator3D),
    I0D(IndicatorN),
}

impl KeyIndicator for Indicator {
    fn from_str(string: &str, is_start: bool) -> Result<Self, &'static str> {
        let res = if string.len() == 1 {
            let a = Indicator1D::from_str(string, is_start)?;
            Indicator::I1D(a)
        } else if string.len() == 2 {
            let a = Indicator2D::from_str(string, is_start)?;
            Indicator::I2D(a)
        } else if string.len() == 2 {
            let a = Indicator3D::from_str(string, is_start)?;
            Indicator::I3D(a)
        } else {
            let a = IndicatorN::from_str(string, is_start)?;
            Indicator::I0D(a)
        };

        Ok(res)
    }

    fn find_in(&self, slice: &[u8], from: usize) -> Option<usize> {
        match self {
            Indicator::I1D(a) => a.find_in(slice, from),
            Indicator::I2D(a) => a.find_in(slice, from),
            Indicator::I3D(a) => a.find_in(slice, from),
            Indicator::I0D(a) => a.find_in(slice, from),
        }
    }

    fn first_char(&self) -> u8 {
        match self {
            Indicator::I1D(a) => a.first_char(),
            Indicator::I2D(a) => a.first_char(),
            Indicator::I3D(a) => a.first_char(),
            Indicator::I0D(a) => a.first_char(),
        }
    }

    fn size(&self) -> usize {
        match self {
            Indicator::I1D(a) => a.size(),
            Indicator::I2D(a) => a.size(),
            Indicator::I3D(a) => a.size(),
            Indicator::I0D(a) => a.size(),
        }
    }
}

impl Display for Indicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Indicator::I1D(i) => {
                write!(f, "{}", std::str::from_utf8(&[i.0]).expect(&format!("Non UTF-8 Key {:?}", &[i.0])) )
            },
            Indicator::I2D(i) => {
                write!(f, "{}", std::str::from_utf8(&[i.0, i.1]).expect(&format!("Non UTF-8 Key {:?}", &[i.0, i.1])) )
            },
            Indicator::I3D(i) => {
                write!(f, "{}", std::str::from_utf8(&[i.0, i.1, i.2]).expect(&format!("Non UTF-8 Key {:?}", &[i.0, i.1, i.2])) )
            },
            Indicator::I0D(i) => {
                write!(f, "{}", std::str::from_utf8(&i.0).expect(&format!("Non UTF-8 Key {:?}", &i.0)) )
            }
        }
    }
}
