struct TimeUnit {
    value: u128,
    singular: String,
    plural: String,
}

impl TimeUnit {
    fn new(value: u128, singular: &str, plural: &str) -> TimeUnit {
        TimeUnit {
            value,
            singular: singular.to_string(),
            plural: plural.to_string(),
        }
    }

    fn to_string(&self) -> String {
        if self.value == 0 {
            return String::new();
        }

        if self.value == 1 {
            return self.value.to_string() + " " + &self.singular;
        }

        self.value.to_string() + " " + &self.plural
    }
}

pub fn format_time(input: u128) -> String {
    if input <= 0 {
        return String::from("No time at all");
    }

    let time_units = vec![
        TimeUnit::new(input / (1000 * 60 * 60 * 24 * 30 * 12), "year", "years"),
        TimeUnit::new(input / (1000 * 60 * 60 * 24 * 30) % 12, "month", "months"),
        TimeUnit::new(input / (1000 * 60 * 60 * 24 * 7) % 4, "week", "weeks"),
        TimeUnit::new(input / (1000 * 60 * 60 * 24) % 7, "day", "days"),
        TimeUnit::new(input / (1000 * 60 * 60) % 24, "hour", "hours"),
        TimeUnit::new(input / (1000 * 60) % 60, "minute", "minutes"),
        TimeUnit::new(input / 1000 % 60, "second", "seconds"),
    ];

    if input < 1000 {
        return input.to_string()
            + " "
            + &(if input == 1 {
                "millisecond"
            } else {
                "milliseconds"
            })
            .to_string();
    }

    let non_zero_units: Vec<String> = time_units
        .iter()
        .filter(|unit| !unit.to_string().is_empty())
        .map(|unit| unit.to_string())
        .collect();

    let length = non_zero_units.len();
    let mut result = String::new();
    for (i, unit) in non_zero_units.iter().enumerate() {
        if i == length - 1 && length > 1 {
            result += " and ";
            result += unit;
        } else if i == length - 2 || length == 1 {
            result += unit;
        } else {
            result += unit;
            result += ", ";
        }
    }

    result
}
