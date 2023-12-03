#[derive(Debug, PartialEq, PartialOrd)]
pub enum Number {
    Integer(i64),
    UnsingedInteger(u64),
    Float(f64),
}

impl Number {
    pub fn parse(sequence: &String) -> Option<Self> {
        if let Ok(integer) = sequence.parse::<u64>() {
            Some(Number::UnsingedInteger(integer))
        } else if let Ok(integer) = sequence.parse::<i64>() {
            Some(Number::Integer(integer))
        } else if let Ok(float) = sequence.parse::<f64>() {
            if float.fract() == 0.0 && float >= u64::MIN as f64 && float <= u64::MAX as f64 {
                Some(Number::UnsingedInteger(float as u64))
            } else if float.fract() == 0.0 && float >= i64::MIN as f64 && float <= i64::MAX as f64 {
                Some(Number::Integer(float as i64))
            } else {
                Some(Number::Float(float))
            }
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Integer(integer) => integer.to_string(),
            Self::UnsingedInteger(integer) => integer.to_string(),
            Self::Float(float) => float.to_string(),
        }
    }
}
