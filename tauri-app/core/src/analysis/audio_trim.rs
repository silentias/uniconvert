use crate::domain::ConvertError;

/// Validate the range once for conversion, progress and size estimation.
pub fn trimmed_duration(
    start: Option<f64>,
    end: Option<f64>,
    duration: Option<f64>,
) -> Result<Option<f64>, ConvertError> {
    let start = start.unwrap_or(0.0);
    if !start.is_finite() || start < 0.0 || end.is_some_and(|end| !end.is_finite() || end <= start)
    {
        return Err(ConvertError::InvalidInput(
            "Начало должно быть неотрицательным, а конец — позже начала фрагмента".into(),
        ));
    }
    let duration = duration.filter(|value| value.is_finite() && *value > 0.0);
    if let Some(duration) = duration {
        // Container durations may differ slightly from decoded sample counts.
        if start >= duration || end.is_some_and(|end| end > duration + 0.1) {
            return Err(ConvertError::InvalidInput(
                "Фрагмент выходит за пределы аудиозаписи".into(),
            ));
        }
    }
    Ok(end.or(duration).map(|end| end - start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_full_and_partial_durations() {
        assert_eq!(
            trimmed_duration(None, None, Some(60.0)).unwrap(),
            Some(60.0)
        );
        assert_eq!(
            trimmed_duration(Some(10.25), Some(20.5), Some(60.0)).unwrap(),
            Some(10.25)
        );
        assert_eq!(
            trimmed_duration(Some(10.0), None, Some(60.0)).unwrap(),
            Some(50.0)
        );
        assert_eq!(
            trimmed_duration(Some(10.0), Some(20.0), None).unwrap(),
            Some(10.0)
        );
        assert_eq!(trimmed_duration(None, None, None).unwrap(), None);
    }

    #[test]
    fn rejects_invalid_or_out_of_bounds_ranges() {
        for (start, end) in [
            (f64::NAN, 2.0),
            (-1.0, 2.0),
            (1.0, f64::INFINITY),
            (2.0, 2.0),
            (3.0, 2.0),
            (60.0, 61.0),
            (0.0, 61.0),
        ] {
            assert!(trimmed_duration(Some(start), Some(end), Some(60.0)).is_err());
        }
    }
}
