//! Every screen draws its text in the interface language. Each surface (a ribbon tab, a task
//! pane, a File page, a dialog) is drawn in English and in Spanish; a label that reads the same
//! in both while the Spanish table has a row for it was drawn without going through `t()`.

use std::collections::BTreeSet;

use egui_kittest::Harness;
use egui_kittest::kittest::NodeT as _;
use lamina_engine::Session;
use serde_json::json;

use crate::backstage::Page;
use crate::dialogs::Dialog;
use crate::i18n::Language;
use crate::state::ViewMode;
use crate::{Services, SlideApp};

/// Everything the accessibility tree names on the current frame.
fn shown(h: &Harness<'_, SlideApp>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for node in h.root().children_recursive() {
        let n = node.accesskit_node();
        for text in [n.label(), n.value()].into_iter().flatten() {
            let text = text.trim();
            if !text.is_empty() {
                out.insert(text.to_owned());
            }
        }
    }
    out
}

/// Puts the app into the state of one surface.
type Setup = Box<dyn Fn(&mut SlideApp)>;

fn surface(language: Language, setup: &dyn Fn(&mut SlideApp)) -> BTreeSet<String> {
    let mut s = Session::with_new();
    s.execute("slide.new", &json!({"layout": "titleAndContent", "title": "Title"})).unwrap();
    let mut app = SlideApp::new(s, Services::default());
    app.ui.language = Some(language);
    setup(&mut app);
    // Only the text is read here; an unnamed field is the accessibility tests' business.
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).with_accessibility_check(false).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    shown(&h)
}

/// Labels that read the same in English and Spanish on this surface.
fn same_in_both(setup: &dyn Fn(&mut SlideApp)) -> Vec<String> {
    let en = surface(Language::En, setup);
    let es = surface(Language::Es, setup);
    en.intersection(&es).cloned().collect()
}

fn surfaces() -> Vec<(String, Setup)> {
    let mut v: Vec<(String, Setup)> = Vec::new();
    for tab in crate::ribbon::TABS {
        v.push((format!("tab {tab}"), Box::new(move |app: &mut SlideApp| app.ui.tab = tab.to_string())));
    }
    for (name, view) in
        [("outline", ViewMode::Outline), ("sorter", ViewMode::Sorter), ("notesPage", ViewMode::NotesPage), ("reading", ViewMode::Reading)]
    {
        v.push((format!("view {name}"), Box::new(move |app: &mut SlideApp| app.ui.view = view)));
    }
    for pane in ["format", "animation", "selection", "comments", "designer", "background", "smartart"] {
        v.push((format!("pane {pane}"), Box::new(move |app: &mut SlideApp| app.ui.pane = Some(pane.to_string()))));
    }
    // What a selection adds: the contextual tabs and the panes that read the selected shape.
    let insert_shape = |app: &mut SlideApp| {
        app.session.execute("shape.insert", &json!({"preset": "rect", "rect": [100.0, 100.0, 300.0, 120.0], "text": "Hello"})).unwrap();
    };
    let tab = "Shape Format";
    v.push((
        format!("shape selected, tab {tab}"),
        Box::new(move |app: &mut SlideApp| {
            insert_shape(app);
            app.ui.tab = tab.to_string();
        }),
    ));
    for pane in ["format", "animation"] {
        for format_tab in ["fill", "effects", "size", "text"] {
            v.push((
                format!("shape selected, pane {pane} {format_tab}"),
                Box::new(move |app: &mut SlideApp| {
                    insert_shape(app);
                    app.ui.pane = Some(pane.to_string());
                    app.ui.format_tab = format_tab.to_string();
                }),
            ));
        }
    }
    for tab in ["Table Design", "Layout"] {
        v.push((
            format!("table selected, tab {tab}"),
            Box::new(move |app: &mut SlideApp| {
                app.session.execute("insert.table", &json!({"rows": 2, "cols": 2})).unwrap();
                app.ui.tab = tab.to_string();
            }),
        ));
    }
    for page in Page::ALL {
        v.push((format!("file {page:?}"), Box::new(move |app: &mut SlideApp| app.open_backstage(page))));
    }
    for name in [
        "about",
        "action",
        "altText",
        "autocorrect",
        "bullets",
        "chartData",
        "close",
        "comment",
        "customShow",
        "dateTime",
        "equation",
        "export",
        "find",
        "gridOptions",
        "hyperlink",
        "language",
        "outline",
        "paragraph",
        "pasteSpecial",
        "preferences",
        "quit",
        "renameSection",
        "replaceFonts",
        "reuseSlides",
        "shortcuts",
        "slideSize",
        "smartart",
        "spelling",
        "table",
        "tabs",
        "themeColors",
        "themeFonts",
        "trim",
        "zoom",
    ] {
        v.push((format!("dialog {name}"), Box::new(move |app: &mut SlideApp| app.dialog = Some(Dialog::new(name)))));
    }
    v
}

/// Labels drawn in English in a Spanish interface although the Spanish table translates them.
fn untranslated(setup: &dyn Fn(&mut SlideApp)) -> Vec<String> {
    same_in_both(setup).into_iter().filter(|label| Language::Es.tr(label) != label.as_str()).collect()
}

/// A screen that draws a label the table translates, without going through `t()`, shows English
/// to everyone who runs Lámina in Spanish.
#[test]
fn spanish_interface_has_no_untranslated_label() {
    let mut failures = Vec::new();
    for (name, setup) in surfaces() {
        let mut labels = untranslated(setup.as_ref());
        labels.retain(|l| !USER_TEXT.contains(&(name.as_str(), l.as_str())));
        if !labels.is_empty() {
            failures.push(format!("{name}: {labels:?}"));
        }
    }
    assert!(failures.is_empty(), "labels drawn in English although Spanish has them:\n{}", failures.join("\n"));
}

/// Prints every surface's labels that read the same in both languages:
/// `cargo test -p lamina-ui-egui --lib -- --ignored --nocapture untranslated_report`.
/// Words that read the same in Spanish, and text that belongs to the document or to the user's
/// machine rather than to the interface (a new deck's names, the example slide, font names).
const SAME_IN_SPANISH: &[&str] = &[
    "Audio",
    "Claude",
    "claude-sonnet-5-5",
    "Color",
    "Color:",
    "Content Placeholder 2",
    "Decimal",
    "General",
    "Horizontal",
    "Inter",
    "Lámina",
    "Lámina 2",
    "Lámina 3",
    "Lámina 4",
    "About Lámina",
    "Material",
    "Normal",
    "Plan",
    "Presentation1",
    "“Presentation1”?",
    "https://",
    "Resize panel",
    "Title 1",
    "Vertical",
    "Zoom",
    "Zoom options",
];

/// Surfaces that list data rather than interface text: the shortcut keys, the installed fonts, the
/// languages in their own name, the autocorrect entries and the About credits.
const DATA_SURFACES: &[&str] = &["dialog about", "dialog autocorrect", "dialog language", "dialog replaceFonts", "dialog shortcuts"];

/// Text the document or the user's settings hold, shown as it was typed: the new deck's author
/// and the title of its first slide in the outline, and the shortcut dialog's key names.
const USER_TEXT: &[(&str, &str)] =
    &[("view outline", "Title"), ("file Options", "Presenter"), ("dialog preferences", "Presenter"), ("dialog shortcuts", "Tab")];

/// A label with words in it that is drawn the same in both languages and that the Spanish table
/// has no row for: it never went through the table at all.
fn without_a_row(setup: &dyn Fn(&mut SlideApp)) -> Vec<String> {
    let has_words = |label: &str| label.split(|c: char| !c.is_ascii_alphabetic()).any(|w| w.len() >= 3);
    same_in_both(setup)
        .into_iter()
        .filter(|label| has_words(label) && !SAME_IN_SPANISH.contains(&label.as_str()))
        .filter(|label| !label.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .collect()
}

/// A screen that draws a label with no Spanish row shows English to everyone who runs Lámina in
/// Spanish; a word that is the same in both languages is listed in `SAME_IN_SPANISH`.
#[test]
fn spanish_interface_has_no_label_without_a_row() {
    let mut failures = Vec::new();
    for (name, setup) in surfaces() {
        if DATA_SURFACES.contains(&name.as_str()) {
            continue;
        }
        let mut labels = without_a_row(setup.as_ref());
        labels.retain(|l| !USER_TEXT.contains(&(name.as_str(), l.as_str())));
        if !labels.is_empty() {
            failures.push(format!("{name}: {labels:?}"));
        }
    }
    assert!(failures.is_empty(), "labels drawn in English with no Spanish row:\n{}", failures.join("\n"));
}

#[test]
#[ignore = "a report, not a check"]
fn untranslated_report() {
    for (name, setup) in surfaces() {
        println!("== {name}");
        for label in same_in_both(setup.as_ref()) {
            let note = if Language::Es.tr(&label) != label.as_str() { " (has a Spanish row)" } else { "" };
            println!("   {label:?}{note}");
        }
    }
}
