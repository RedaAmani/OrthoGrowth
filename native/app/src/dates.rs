//! Dates ISO (AAAA-MM-JJ) et âge décimal, comme le plugin (jours / 365,2425).

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Nombre de jours depuis 1970-01-01, ou None si la date n'est pas valide.
pub fn parse_days(s: &str) -> Option<i64> {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' { return None; }
    let y: i64 = s[0..4].parse().ok()?;
    let m: i64 = s[5..7].parse().ok()?;
    let d: i64 = s[8..10].parse().ok()?;
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = match m { 1 | 3 | 5 | 7 | 8 | 10 | 12 => 31, 4 | 6 | 9 | 11 => 30, 2 => if leap { 29 } else { 28 }, _ => return None };
    if d < 1 || d > dim || y < 1 { return None; }
    Some(days_from_civil(y, m, d))
}

/// Âge en années décimales entre naissance et examen (None si une date est invalide ou l'ordre incohérent).
pub fn decimal_age(dob: &str, exam: &str) -> Option<f64> {
    let (b, e) = (parse_days(dob)?, parse_days(exam)?);
    if e < b { return None; }
    Some((e - b) as f64 / 365.2425)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates() {
        assert_eq!(parse_days("1970-01-01"), Some(0));
        assert_eq!(parse_days("2024-02-29").is_some(), true);
        assert_eq!(parse_days("2023-02-29"), None);
        assert_eq!(parse_days("2023-13-01"), None);
        let a = decimal_age("2016-01-01", "2026-05-01").unwrap();
        assert!((a - 10.33).abs() < 0.01);
        assert_eq!(decimal_age("2026-01-01", "2016-01-01"), None);
    }
}
