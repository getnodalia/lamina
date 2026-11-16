//! Interface language selection and the first translated File and Preferences strings.
//! Other interface strings still fall back to English.
use serde::{Deserialize, Serialize};

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

    pub fn tr(self, english: &'static str) -> &'static str {
        if self != Self::Es {
            return english;
        }
        match english {
            "Home" => "Inicio",
            "New" => "Nuevo",
            "Open" => "Abrir",
            "Info" => "Información",
            "Save As" => "Guardar como",
            "Save" => "Guardar",
            "Close" => "Cerrar",
            "Print" => "Imprimir",
            "Export" => "Exportar",
            "About" => "Acerca de",
            "Preferences" => "Preferencias",
            "Back" => "Volver",
            "General" => "General",
            "Editing" => "Edición",
            "Interface language:" => "Idioma de la interfaz:",
            "Automatic" => "Automático",
            "Display theme:" => "Tema de la interfaz:",
            "Light" => "Claro",
            "Dark" => "Oscuro",
            "Window controls:" => "Controles de ventana:",
            "Use the desktop's" => "Los del escritorio",
            "Show the Start screen when Lámina starts" => "Mostrar la pantalla de inicio al abrir Lámina",
            "Interface size:" => "Tamaño de la interfaz:",
            "Interface size" => "Tamaño de la interfaz",
            "Measurement units:" => "Unidades de medida:",
            "Inches" => "Pulgadas",
            "Centimeters" => "Centímetros",
            "Show smart guides" => "Mostrar guías inteligentes",
            "Snap objects to grid" => "Ajustar objetos a la cuadrícula",
            "Replace straight quotes with smart quotes" => "Sustituir comillas rectas por tipográficas",
            "AutoCorrect as you type" => "Autocorrección al escribir",
            "AutoCorrect Options…" => "Opciones de autocorrección…",
            "Your name (comments):" => "Tu nombre (comentarios):",
            _ => english,
        }
    }
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
        assert_eq!(Language::Es.tr("not translated"), "not translated");
    }
}
