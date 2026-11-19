//! Date and time fields (`a:fld type="datetime…"`): the text each field type shows for a moment,
//! and the local clock.
//!
//! The field types and their formats follow ECMA-376 Part 1, §21.1.2.2.4 (`fld`): `datetime1`
//! to `datetime13`, written as in PowerPoint's English (United States) Date and Time list.
//! `datetime` and `datetimeFigureOut` show the language's short date. Numeric dates follow the
//! run's language; formats with month or day names, or AM/PM, are computed for English, Spanish,
//! French, German, Italian and Portuguese, so other languages keep the text the file was saved
//! with.

/// A local date and time: what the clock read, or a fixed moment (tests, deterministic output).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    /// 1–31.
    pub day: u32,
    /// 0–23.
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl DateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        DateTime { year, month, day, hour, minute, second }
    }

    /// The local date and time now (the user's time zone); `None` where there is no clock (wasm).
    pub fn now_local() -> Option<DateTime> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use chrono::{Datelike, Timelike};
            let n = chrono::Local::now();
            Some(DateTime::new(n.year(), n.month(), n.day(), n.hour(), n.minute(), n.second()))
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }

    /// A real calendar date and time of day.
    fn is_valid(&self) -> bool {
        let date_ok = (1..=9999).contains(&self.year) && chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day).is_some();
        date_ok && self.hour < 24 && self.minute < 60 && self.second < 61
    }

    /// Day of the week, 0 = Sunday.
    fn weekday(&self) -> usize {
        match chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day) {
            // `chrono::Weekday::num_days_from_sunday` is 0 = Sunday.
            Some(d) => chrono::Datelike::weekday(&d).num_days_from_sunday() as usize,
            None => 0,
        }
    }
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

/// Month and day names and the date layout of a language other than English.
struct Names {
    months: [&'static str; 12],
    /// Abbreviated months, as written between a day and a year ("9 oct 2026").
    mons: [&'static str; 12],
    /// Sunday first.
    days: [&'static str; 7],
    /// What joins the day, month and year in a long date: `" de "` in Spanish and Portuguese.
    of: &'static str,
    /// Written after a weekday name.
    weekday_sep: &'static str,
    /// German writes an ordinal dot after the day ("9. Oktober").
    day_dot: bool,
    /// `a. m.` / `p. m.` where the language keeps the 12-hour clock for times, `None` for 24-hour.
    ampm: Option<(&'static str, &'static str)>,
}

const ES: Names = Names {
    months: ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"],
    mons: ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"],
    days: ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"],
    of: " de ",
    weekday_sep: ", ",
    day_dot: false,
    ampm: Some(("a. m.", "p. m.")),
};
const FR: Names = Names {
    months: ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"],
    mons: ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."],
    days: ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"],
    of: " ",
    weekday_sep: " ",
    day_dot: false,
    ampm: None,
};
const DE: Names = Names {
    months: ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"],
    mons: ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"],
    days: ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"],
    of: " ",
    weekday_sep: ", ",
    day_dot: true,
    ampm: None,
};
const IT: Names = Names {
    months: ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"],
    mons: ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"],
    days: ["domenica", "lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato"],
    of: " ",
    weekday_sep: " ",
    day_dot: false,
    ampm: None,
};
const PT: Names = Names {
    months: ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"],
    mons: ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"],
    days: ["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"],
    of: " de ",
    weekday_sep: ", ",
    day_dot: false,
    ampm: None,
};

/// The names of the non-English language `lang` (lower case) that has them.
fn names_of(lang: &str) -> Option<&'static Names> {
    match lang.split(['-', '_']).next().unwrap_or("") {
        "es" => Some(&ES),
        "fr" => Some(&FR),
        "de" => Some(&DE),
        "it" => Some(&IT),
        "pt" => Some(&PT),
        _ => None,
    }
}

/// The long and clock formats of a language with [`Names`].
fn named_other(kind: &str, n: &Names, now: DateTime, short: &str) -> Option<String> {
    let month = n.months.get(now.month.checked_sub(1)? as usize)?;
    let mon = n.mons.get(now.month.checked_sub(1)? as usize)?;
    let day = n.days.get(now.weekday())?;
    let (d, y, yy) = (now.day, now.year, now.year.rem_euclid(100));
    let dot = if n.day_dot { "." } else { "" };
    let (of, sep) = (n.of, n.weekday_sep);
    let (hour, mi, s) = (now.hour, now.minute, now.second);
    let h12 = match hour % 12 {
        0 => 12,
        h => h,
    };
    // Times: the 12-hour clock with the language's AM/PM marks, or the 24-hour clock.
    let clock = |secs: bool| match n.ampm {
        Some((am, pm)) => {
            let mark = if hour < 12 { am } else { pm };
            if secs { format!("{h12}:{mi:02}:{s:02} {mark}") } else { format!("{h12}:{mi:02} {mark}") }
        }
        None if secs => format!("{hour:02}:{mi:02}:{s:02}"),
        None => format!("{hour:02}:{mi:02}"),
    };
    Some(match kind {
        "datetime2" => format!("{day}{sep}{d}{dot}{of}{month}{of}{y}"),
        "datetime3" => format!("{d}{dot}{of}{month}{of}{y}"),
        "datetime4" => format!("{d}{dot} {mon} {y}"),
        "datetime5" => format!("{d}-{mon}-{yy:02}"),
        "datetime6" => format!("{month} {yy:02}"),
        "datetime7" => format!("{mon}-{yy:02}"),
        "datetime8" => format!("{short} {}", clock(false)),
        "datetime9" => format!("{short} {}", clock(true)),
        "datetime12" => clock(false),
        "datetime13" => clock(true),
        _ => return None,
    })
}

/// The date languages Date & Time offers, from a BCP 47 tag in any case (`en_gb`, `es`, `PT-BR`):
/// `en-US`, `en-GB`, `es-ES`, `es-MX`, `fr-FR`, `de-DE`, `it-IT`, `pt-BR`, `ja-JP` or `sv-SE`.
pub fn canonical_language(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().replace('_', "-").as_str() {
        "en" | "en-us" => Some("en-US"),
        "en-gb" => Some("en-GB"),
        "es" | "es-es" => Some("es-ES"),
        "es-mx" => Some("es-MX"),
        "fr" | "fr-fr" => Some("fr-FR"),
        "de" | "de-de" => Some("de-DE"),
        "it" | "it-it" => Some("it-IT"),
        "pt" | "pt-br" => Some("pt-BR"),
        "ja" | "ja-jp" => Some("ja-JP"),
        "sv" | "sv-se" => Some("sv-SE"),
        _ => None,
    }
}

/// Is field `kind` a date/time field?
pub fn is_date_field(kind: &str) -> bool {
    kind.starts_with("datetime")
}

/// The date and time field types PowerPoint's Header and Footer and Insert ▸ Date & Time offer,
/// in the order the dialogs list them. `datetime` and `datetimeFigureOut` are read as `datetime1`
/// (see [`field_text`]) but are not offered as a choice.
pub const DATE_KINDS: [&str; 13] = [
    "datetime1",
    "datetime2",
    "datetime3",
    "datetime4",
    "datetime5",
    "datetime6",
    "datetime7",
    "datetime8",
    "datetime9",
    "datetime10",
    "datetime11",
    "datetime12",
    "datetime13",
];

/// The text date/time field `kind` shows at `now` in a run of language `lang` (BCP 47, e.g.
/// `en-US`). `None` when `kind` isn't a date/time type this knows, `now` isn't a real date, or
/// the format needs names in a language other than English: the field keeps its saved text.
pub fn field_text(kind: &str, lang: Option<&str>, now: DateTime) -> Option<String> {
    if !now.is_valid() {
        return None;
    }
    let lang = lang.map(str::trim).filter(|l| !l.is_empty()).unwrap_or("en-US").to_ascii_lowercase();
    let english = lang == "en" || lang.starts_with("en-");
    let month = MONTHS.get(now.month.checked_sub(1)? as usize)?;
    let mon = month.get(..3)?;
    let day = DAYS.get(now.weekday())?;
    let (d, y) = (now.day, now.year);
    let yy = y.rem_euclid(100);
    let h12 = match now.hour % 12 {
        0 => 12,
        h => h,
    };
    let ampm = if now.hour < 12 { "AM" } else { "PM" };
    let (mi, s) = (now.minute, now.second);
    let short = short_date(&lang, now);
    // British-style English writes the day before the month ("9 October 2026"). The punctuation
    // follows PowerPoint on Windows per locale: Irish English has no comma after the weekday,
    // British and Australian English put one after the month in `datetime3`, and South African
    // English writes the weekday form day first too.
    let day_first = matches!(lang.as_str(), "en-gb" | "en-au" | "en-nz" | "en-ie" | "en-in");
    let named = |text: String| english.then_some(text);
    if !english && let Some(n) = names_of(&lang) {
        // The numeric formats below are the same in every language; the named ones come from here.
        if let Some(text) = named_other(kind, n, now, &short) {
            return Some(text);
        }
    }
    match kind {
        "datetime" | "datetimeFigureOut" | "datetime1" => Some(short),
        "datetime2" if lang == "en-ie" => named(format!("{day} {d} {month} {y}")),
        "datetime2" if day_first || lang == "en-za" => named(format!("{day}, {d} {month} {y}")),
        "datetime2" => named(format!("{day}, {month} {d}, {y}")),
        "datetime3" if matches!(lang.as_str(), "en-gb" | "en-au") => named(format!("{d} {month}, {y}")),
        "datetime3" => named(format!("{d} {month} {y}")),
        "datetime4" if day_first => named(format!("{d} {month} {y}")),
        "datetime4" => named(format!("{month} {d}, {y}")),
        "datetime5" => named(format!("{d}-{mon}-{yy:02}")),
        "datetime6" => named(format!("{month} {yy:02}")),
        "datetime7" => named(format!("{mon}-{yy:02}")),
        "datetime8" => named(format!("{short} {h12}:{mi:02} {ampm}")),
        "datetime9" => named(format!("{short} {h12}:{mi:02}:{s:02} {ampm}")),
        "datetime10" => Some(format!("{}:{mi:02}", now.hour)),
        "datetime11" => Some(format!("{}:{mi:02}:{s:02}", now.hour)),
        "datetime12" => named(format!("{h12}:{mi:02} {ampm}")),
        "datetime13" => named(format!("{h12}:{mi:02}:{s:02} {ampm}")),
        _ => None,
    }
}

/// The short (numeric) date of language `lang` (lower case), US order unless the language is
/// known to write its dates another way.
fn short_date(lang: &str, t: DateTime) -> String {
    let (y, m, d) = (t.year, t.month, t.day);
    let primary = lang.split(['-', '_']).next().unwrap_or("");
    match lang {
        "en-gb" | "en-au" | "en-nz" | "en-ie" | "en-in" => return format!("{d:02}/{m:02}/{y}"),
        "en-ca" | "fr-ca" | "en-za" => return format!("{y}-{m:02}-{d:02}"),
        _ => {}
    }
    match primary {
        "fr" | "es" | "it" | "pt" | "el" | "ca" | "vi" => format!("{d:02}/{m:02}/{y}"),
        "de" | "ru" | "pl" | "cs" | "sk" | "tr" | "fi" | "nb" | "nn" | "no" | "da" | "uk" | "ro" => format!("{d:02}.{m:02}.{y}"),
        "nl" => format!("{d}-{m}-{y}"),
        "ja" | "zh" => format!("{y}/{m}/{d}"),
        "ko" | "sv" | "lt" => format!("{y}-{m:02}-{d:02}"),
        _ => format!("{m}/{d}/{y}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRI: DateTime = DateTime { year: 2026, month: 10, day: 9, hour: 20, minute: 5, second: 7 };

    #[test]
    fn english_formats_of_every_type() {
        let f = |k: &str| field_text(k, Some("en-US"), FRI);
        assert_eq!(f("datetime1").as_deref(), Some("10/9/2026"));
        assert_eq!(f("datetime2").as_deref(), Some("Friday, October 9, 2026"));
        assert_eq!(f("datetime3").as_deref(), Some("9 October 2026"));
        assert_eq!(f("datetime4").as_deref(), Some("October 9, 2026"));
        assert_eq!(f("datetime5").as_deref(), Some("9-Oct-26"));
        assert_eq!(f("datetime6").as_deref(), Some("October 26"));
        assert_eq!(f("datetime7").as_deref(), Some("Oct-26"));
        assert_eq!(f("datetime8").as_deref(), Some("10/9/2026 8:05 PM"));
        assert_eq!(f("datetime9").as_deref(), Some("10/9/2026 8:05:07 PM"));
        assert_eq!(f("datetime10").as_deref(), Some("20:05"));
        assert_eq!(f("datetime11").as_deref(), Some("20:05:07"));
        assert_eq!(f("datetime12").as_deref(), Some("8:05 PM"));
        assert_eq!(f("datetime13").as_deref(), Some("8:05:07 PM"));
        assert_eq!(f("datetimeFigureOut").as_deref(), Some("10/9/2026"));
        assert_eq!(f("datetime").as_deref(), Some("10/9/2026"));
        // No language: US English. Midnight and noon are 12 on the clock.
        assert_eq!(field_text("datetime12", None, DateTime::new(2024, 2, 29, 0, 0, 0)).as_deref(), Some("12:00 AM"));
        assert_eq!(field_text("datetime12", None, DateTime::new(2024, 2, 29, 12, 30, 0)).as_deref(), Some("12:30 PM"));
        assert_eq!(field_text("datetime2", None, DateTime::new(2000, 1, 1, 0, 0, 0)).as_deref(), Some("Saturday, January 1, 2000"));
    }

    #[test]
    fn language_orders_numeric_dates_and_keeps_names_english_only() {
        assert_eq!(field_text("datetime1", Some("en-GB"), FRI).as_deref(), Some("09/10/2026"));
        assert_eq!(field_text("datetimeFigureOut", Some("de-DE"), FRI).as_deref(), Some("09.10.2026"));
        assert_eq!(field_text("datetime1", Some("ja-JP"), FRI).as_deref(), Some("2026/10/9"));
        assert_eq!(field_text("datetime1", Some("sv-SE"), FRI).as_deref(), Some("2026-10-09"));
        assert_eq!(field_text("datetime10", Some("fr-FR"), FRI).as_deref(), Some("20:05"));
        // Month names in a language without a table: not computed (the field keeps its saved text).
        assert_eq!(field_text("datetime2", Some("ja-JP"), FRI), None);
        assert_eq!(field_text("datetime12", Some("sv-SE"), FRI), None);
        assert_eq!(field_text("datetime4", Some("en-GB"), FRI).as_deref(), Some("9 October 2026"));
        assert_eq!(field_text("datetime2", Some("en-GB"), FRI).as_deref(), Some("Friday, 9 October 2026"));
        assert_eq!(field_text("datetime4", Some("en-AU"), FRI).as_deref(), Some("9 October 2026"));
        assert_eq!(field_text("datetime4", Some("en-US"), FRI).as_deref(), Some("October 9, 2026"));
    }

    /// The long English dates as PowerPoint on Windows shows them on Tuesday 27 October 2026, read
    /// per run language (datetime2, datetime3, datetime4).
    #[test]
    fn long_english_dates_follow_each_locale() {
        let tue = DateTime::new(2026, 10, 27, 9, 0, 0);
        let rows = [
            ("en-US", "Tuesday, October 27, 2026", "27 October 2026", "October 27, 2026"),
            ("en-GB", "Tuesday, 27 October 2026", "27 October, 2026", "27 October 2026"),
            ("en-AU", "Tuesday, 27 October 2026", "27 October, 2026", "27 October 2026"),
            ("en-NZ", "Tuesday, 27 October 2026", "27 October 2026", "27 October 2026"),
            ("en-IE", "Tuesday 27 October 2026", "27 October 2026", "27 October 2026"),
            ("en-IN", "Tuesday, 27 October 2026", "27 October 2026", "27 October 2026"),
            ("en-CA", "Tuesday, October 27, 2026", "27 October 2026", "October 27, 2026"),
        ];
        for (lang, d2, d3, d4) in rows {
            assert_eq!(field_text("datetime2", Some(lang), tue).as_deref(), Some(d2), "{lang}");
            assert_eq!(field_text("datetime3", Some(lang), tue).as_deref(), Some(d3), "{lang}");
            assert_eq!(field_text("datetime4", Some(lang), tue).as_deref(), Some(d4), "{lang}");
        }
        assert_eq!(field_text("datetime2", Some("en-ZA"), tue).as_deref(), Some("Tuesday, 27 October 2026"));
    }

    /// Friday 9 October 2026, 20:05:07, in each language that has names.
    #[test]
    fn named_formats_in_spanish_french_german_italian_and_portuguese() {
        let rows = [
            ("es-ES", ["viernes, 9 de octubre de 2026", "9 de octubre de 2026", "9 oct 2026", "9-oct-26", "octubre 26", "oct-26"]),
            ("es-MX", ["viernes, 9 de octubre de 2026", "9 de octubre de 2026", "9 oct 2026", "9-oct-26", "octubre 26", "oct-26"]),
            ("fr-FR", ["vendredi 9 octobre 2026", "9 octobre 2026", "9 oct. 2026", "9-oct.-26", "octobre 26", "oct.-26"]),
            ("de-DE", ["Freitag, 9. Oktober 2026", "9. Oktober 2026", "9. Okt 2026", "9-Okt-26", "Oktober 26", "Okt-26"]),
            ("it-IT", ["venerdì 9 ottobre 2026", "9 ottobre 2026", "9 ott 2026", "9-ott-26", "ottobre 26", "ott-26"]),
            ("pt-BR", ["sexta-feira, 9 de outubro de 2026", "9 de outubro de 2026", "9 out 2026", "9-out-26", "outubro 26", "out-26"]),
        ];
        for (lang, want) in rows {
            for (kind, want) in ["datetime2", "datetime3", "datetime4", "datetime5", "datetime6", "datetime7"].into_iter().zip(want) {
                assert_eq!(field_text(kind, Some(lang), FRI).as_deref(), Some(want), "{lang} {kind}");
            }
        }
        let f = |lang: &str, k: &str| field_text(k, Some(lang), FRI);
        assert_eq!(f("es-ES", "datetime8").as_deref(), Some("09/10/2026 8:05 p. m."));
        assert_eq!(f("es-ES", "datetime13").as_deref(), Some("8:05:07 p. m."));
        assert_eq!(f("fr-FR", "datetime9").as_deref(), Some("09/10/2026 20:05:07"));
        assert_eq!(f("de-DE", "datetime12").as_deref(), Some("20:05"));
        assert_eq!(f("it-IT", "datetime8").as_deref(), Some("09/10/2026 20:05"));
        // Midnight reads 12 a. m. on the 12-hour Spanish clock, and the weekday follows the date.
        let midnight = DateTime::new(2026, 10, 11, 0, 0, 0);
        assert_eq!(field_text("datetime12", Some("es-ES"), midnight).as_deref(), Some("12:00 a. m."));
        assert_eq!(field_text("datetime2", Some("es-ES"), midnight).as_deref(), Some("domingo, 11 de octubre de 2026"));
        // Other languages still keep their saved text for named formats, and every month is named.
        assert_eq!(field_text("datetime3", Some("ja-JP"), FRI), None);
        for month in 1..=12 {
            assert!(field_text("datetime3", Some("de-DE"), DateTime::new(2026, month, 1, 0, 0, 0)).is_some(), "{month}");
        }
    }

    #[test]
    fn languages_are_read_in_any_case_and_unknown_ones_are_refused() {
        for (raw, want) in
            [("EN", "en-US"), ("en_gb", "en-GB"), (" es ", "es-ES"), ("es-MX", "es-MX"), ("PT-br", "pt-BR"), ("de", "de-DE"), ("sv_SE", "sv-SE")]
        {
            assert_eq!(canonical_language(raw), Some(want), "{raw}");
        }
        for bad in ["", "xx-YY", "es-AR", "english", "\u{0}"] {
            assert_eq!(canonical_language(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn unknown_types_and_impossible_dates_give_nothing() {
        assert_eq!(field_text("slidenum", None, FRI), None);
        assert_eq!(field_text("datetime14", None, FRI), None);
        for bad in [
            DateTime::new(2026, 0, 9, 0, 0, 0),
            DateTime::new(2026, 13, 9, 0, 0, 0),
            DateTime::new(2026, 2, 29, 0, 0, 0),
            DateTime::new(2026, 4, 31, 0, 0, 0),
            DateTime::new(2026, 1, 1, 24, 0, 0),
            DateTime::new(i32::MIN, 1, 1, 0, 0, 0),
            DateTime::new(i32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX),
        ] {
            assert_eq!(field_text("datetime2", None, bad), None, "{bad:?}");
        }
        assert!(is_date_field("datetimeFigureOut") && !is_date_field("slidenum"));
    }

    #[test]
    fn the_clock_reads_a_real_date() {
        if let Some(now) = DateTime::now_local() {
            assert!(field_text("datetime1", None, now).is_some(), "{now:?}");
        }
    }
}
