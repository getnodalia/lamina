use super::{DATE_KINDS, Dialog, SlideApp, Tokens, Ui, buttons, check, json};

const DATE_LANGS: &[(&str, &str)] = &[
    ("en-US", "English (United States)"),
    ("en-GB", "English (United Kingdom)"),
    ("es-ES", "Spanish (Spain)"),
    ("es-MX", "Spanish (Mexico)"),
    ("fr-FR", "French (France)"),
    ("de-DE", "German (Germany)"),
    ("it-IT", "Italian (Italy)"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("ja-JP", "Japanese"),
    ("sv-SE", "Swedish"),
];

/// The date language a new dialog starts in: the interface language's.
pub(super) fn default_date_language() -> &'static str {
    match crate::i18n::current() {
        crate::i18n::Language::Es => "es-ES",
        crate::i18n::Language::En => "en-US",
    }
}

/// Samples `field_text` can compute for `lang`, each shown once. Month names and AM/PM are
/// computed for English, Spanish, French, German, Italian and Portuguese; other languages keep
/// the numeric date and the 24-hour times.
pub(super) fn date_rows(lang: &str) -> Vec<(&'static str, String)> {
    let now = lamina_text::datetime::DateTime::now_local();
    let mut rows: Vec<(&'static str, String)> = Vec::new();
    for kind in DATE_KINDS {
        if let Some(now) = now
            && let Some(sample) = lamina_text::datetime::field_text(kind, Some(lang), now)
            && !rows.iter().any(|(_, s)| *s == sample)
        {
            rows.push((kind, sample));
        }
    }
    if rows.is_empty() {
        rows.push(("datetime1", "datetime1".into()));
    }
    rows
}

/// Render language and format controls; OK applies `insert.dateTime`, and the return value says
/// whether the dialog should close.
pub(super) fn date_time_dialog(app: &mut SlideApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    ui.set_min_width(440.0);
    ui.label(crate::i18n::t("Available formats:"));
    let lang = d.get("language", default_date_language());
    let rows = date_rows(&lang);
    if !rows.iter().any(|(k, _)| *k == d.get("format", "datetime1"))
        && let Some((k, _)) = rows.first()
    {
        d.fields.insert("format".into(), (*k).to_string());
    }
    let format = d.get("format", "datetime1");
    egui::ScrollArea::vertical().max_height(280.0).min_scrolled_height(220.0).auto_shrink([false, false]).show(ui, |ui| {
        for (kind, sample) in &rows {
            if ui.selectable_label(format == *kind, crate::i18n::t(sample)).clicked() {
                d.fields.insert("format".into(), (*kind).to_string());
            }
        }
    });
    let lang_label = crate::i18n::t(DATE_LANGS.iter().find(|(k, _)| *k == lang).map_or("English (United States)", |(_, l)| *l));
    ui.horizontal(|ui| {
        ui.label(crate::i18n::t("Language:"));
        let opened = nodalia_ui::ComboBox::from_id_salt("date-lang").selected_text(lang_label).width(240.0).show_ui(ui, |ui| {
            for (k, l) in DATE_LANGS {
                if ui.selectable_label(lang == *k, crate::i18n::t(l)).clicked() {
                    d.fields.insert("language".into(), (*k).to_string());
                }
            }
        });
        let shown = lang_label.to_string();
        opened.response.widget_info(move || {
            let mut info = egui::WidgetInfo::labeled(egui::Role::ComboBox, true, crate::i18n::t("Language"));
            info.current_text_value = Some(shown.clone());
            info
        });
    });
    ui.label(
        egui::RichText::new(crate::i18n::t("Month names are listed for English, Spanish, French, German, Italian and Portuguese. Other languages list the numeric date and 24-hour time."))
            .size(11.0)
            .color(Tokens::get(ui.ctx()).text_dim),
    );
    let update = check(ui, d, "Update automatically", "update", true);
    let (ok, cancel) = buttons(ui, "OK");
    if ok {
        let lang = d.get("language", default_date_language());
        let mut format = d.get("format", "datetime1");
        let probe = lamina_text::datetime::DateTime::new(2026, 10, 9, 20, 5, 7);
        if lamina_text::datetime::field_text(&format, Some(&lang), probe).is_none() {
            format = "datetime1".into();
        }
        return app.run("insert.dateTime", json!({"format": format, "update": update, "language": lang})).is_ok();
    }
    cancel
}
