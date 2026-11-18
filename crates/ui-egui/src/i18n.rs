//! Interface language selection and translation. `t` translates an English interface string into
//! the language of the frame being drawn; the Spanish lives in `es.rs`, sorted by its English.
use serde::{Deserialize, Serialize};

mod es;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Es,
}

impl Language {
    /// Resolve an interface locale; unsupported locales use English.
    pub fn from_locale(locale: &str) -> Self {
        let language = locale.split(['-', '_', '.']).next().unwrap_or("");
        if language.eq_ignore_ascii_case("es") { Self::Es } else { Self::En }
    }

    /// Use the host locale when no explicit preference has been saved.
    pub fn system() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            static LANGUAGE: std::sync::OnceLock<Language> = std::sync::OnceLock::new();
            *LANGUAGE.get_or_init(|| Self::from_locale(&sys_locale::get_locale().unwrap_or_default()))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::En
        }
    }

    /// The Spanish for an English interface string, or the English when there is none.
    pub fn tr(self, english: &str) -> &str {
        if self != Self::Es {
            return english;
        }
        static SPANISH: std::sync::OnceLock<std::collections::HashMap<&'static str, &'static str>> = std::sync::OnceLock::new();
        SPANISH.get_or_init(|| es::TABLE.iter().copied().collect()).get(english).copied().unwrap_or(english)
    }
}

thread_local! {
    /// The language this thread's frame is drawn in. Each test harness runs on its own thread, so
    /// one test drawing in Spanish never changes another's labels.
    static CURRENT: std::cell::Cell<Language> = const { std::cell::Cell::new(Language::En) };
}

/// Draw the rest of this frame in `language`. Called once at the start of every UI pass.
pub fn set_current(language: Language) {
    CURRENT.with(|c| c.set(language));
}

/// The language the current frame is drawn in.
pub fn current() -> Language {
    CURRENT.with(std::cell::Cell::get)
}

/// Translate an English interface string into the current frame's language.
/// "3 of 8" in the frame's language.
pub fn of(n: usize, total: usize) -> String {
    match current() {
        Language::Es => format!("{n} de {total}"),
        Language::En => format!("{n} of {total}"),
    }
}

/// "Slide 3 of 8" in the frame's language.
pub fn slide_of(n: usize, total: usize) -> String {
    format!("{} {}", t("Slide"), of(n, total))
}

pub fn t(english: &str) -> &str {
    current().tr(english)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::Language;

    #[test]
    fn saved_language_round_trips_and_old_settings_default_to_automatic() {
        let old: crate::UiState = serde_json::from_str("{}").unwrap();
        assert_eq!(old.language, None);
        let ui = crate::UiState { language: Some(Language::Es), ..Default::default() };
        let restored: crate::UiState = serde_json::from_str(&serde_json::to_string(&ui).unwrap()).unwrap();
        assert_eq!(restored.language, Some(Language::Es));
    }

    #[test]
    fn locale_and_fallback() {
        assert_eq!(Language::from_locale("es_ES.UTF-8"), Language::Es);
        assert_eq!(Language::from_locale("ES-MX"), Language::Es);
        assert_eq!(Language::from_locale("en-US"), Language::En);
        assert_eq!(Language::Es.tr("Preferences"), "Preferencias");
        assert_eq!(Language::Es.tr("Transitions"), "Transiciones");
        assert_eq!(Language::En.tr("Transitions"), "Transitions");
        assert_eq!(Language::Es.tr("not translated"), "not translated");
    }

    #[test]
    fn spanish_ribbon_labels_keep_the_original_tab_identity() {
        use egui_kittest::{Harness, kittest::Queryable};

        let mut app = crate::SlideApp::new(lamina_engine::Session::with_new(), crate::Services::default());
        app.ui.language = Some(Language::Es);
        let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(
            |ui, app: &mut crate::SlideApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
        h.run_steps(3);
        h.get_by_label("Insertar").click();
        h.run_steps(3);
        assert_eq!(h.state().ui.tab, "Insert");
        assert!(h.query_by_label("Archivo").is_some());
    }

    #[test]
    fn spanish_table_is_sorted_unique_and_keeps_the_shape_of_each_label() {
        for pair in super::es::TABLE.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{:?} must come before {:?}, once", pair[1].0, pair[0].0);
        }
        for (en, es) in super::es::TABLE {
            assert!(!es.trim().is_empty() && en != es, "{en:?}");
            assert_eq!(en.matches('\n').count(), es.matches('\n').count(), "{en:?} -> {es:?}: ribbon labels keep their two lines");
            assert_eq!(en.ends_with('…'), es.ends_with('…'), "{en:?} -> {es:?}: a dialog stays marked with …");
            assert_eq!(en.ends_with(':'), es.ends_with(':'), "{en:?} -> {es:?}");
        }
    }

    #[test]
    fn the_home_tab_and_its_buttons_read_in_spanish() {
        use egui_kittest::{Harness, kittest::Queryable};

        let mut app = crate::SlideApp::new(lamina_engine::Session::with_new(), crate::Services::default());
        app.ui.language = Some(Language::Es);
        let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(
            |ui, app: &mut crate::SlideApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
        h.run_steps(3);
        assert!(h.query_all_by_label_contains("Pegar").next().is_some());
        assert!(h.query_all_by_label_contains("Paste").next().is_none());
        assert!(h.query_all_by_label_contains("Nueva").next().is_some());
        assert_eq!(super::current(), Language::Es);
        super::set_current(Language::En);
        assert_eq!(super::t("Paste"), "Paste");
        assert_eq!(super::slide_of(3, 8), "Slide 3 of 8");
        super::set_current(Language::Es);
        assert_eq!(super::slide_of(3, 8), "Diapositiva 3 de 8");
    }
}
