//! Insert › Chart from Spreadsheet: pick an `.xlsx`, `.csv` or `.tsv`, name the sheet and range,
//! and the chart is drawn from those cells and remembers where they came from.

use super::{Dialog, SlideApp, Ui, buttons, check, field, json};

/// The file's name for the dialog's label: the last path segment.
fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

pub(super) fn chart_from_sheet_dialog(app: &mut SlideApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    let mut path = d.fields.get("path").cloned().unwrap_or_default();
    ui.horizontal(|ui| {
        let l = ui.add_sized(egui::vec2(120.0, 20.0), egui::Label::new(crate::i18n::t("Spreadsheet file:")));
        ui.add(egui::TextEdit::singleline(&mut path).desired_width(180.0)).labelled_by(l.id);
        if app.services.pick_open.is_some()
            && ui.button(crate::i18n::t("Choose…")).clicked()
            && let Some(picked) = app.services.pick_open.as_mut().and_then(|p| p("sheet"))
        {
            path = picked;
        }
    });
    d.fields.insert("path".into(), path.clone());
    if !path.trim().is_empty() {
        ui.weak(file_name(path.trim()));
    }
    field(ui, d, "Sheet (optional):", "sheet", "");
    field(ui, d, "Range, such as A1:D9 (optional):", "range", "");
    check(ui, d, "Each row is a series", "byRows", false);

    let kinds = lamina_model::chart::ChartType::MAIN;
    let chosen = d.fields.get("type").cloned().unwrap_or_else(|| "Column".into());
    let label = kinds.iter().find(|k| format!("{k:?}") == chosen).map_or("Column", |k| k.label());
    ui.horizontal(|ui| {
        let l = ui.label(crate::i18n::t("Chart type:"));
        let combo = nodalia_ui::ComboBox::from_id_salt("sheetchart_type").selected_text(crate::i18n::t(label)).show_ui(ui, |ui| {
            for k in kinds {
                if ui.selectable_label(format!("{k:?}") == chosen, crate::i18n::t(k.label())).clicked() {
                    d.fields.insert("type".into(), format!("{k:?}"));
                }
            }
        });
        let _ = combo.response.labelled_by(l.id);
    });
    if let Some(e) = d.fields.get("error").filter(|e| !e.is_empty()) {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    let (ok, cancel) = buttons(ui, "Insert");
    if ok {
        if path.trim().is_empty() {
            d.fields.insert("error".into(), crate::i18n::t("Choose a spreadsheet file first.").to_string());
            return false;
        }
        let get = |k: &str| d.fields.get(k).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let mut p =
            json!({"path": path.trim(), "type": chosen, "series": if d.fields.get("byRows").is_some_and(|v| v == "1") { "rows" } else { "columns" }});
        if let Some(s) = get("sheet") {
            p["sheet"] = s.into();
        }
        if let Some(r) = get("range") {
            p["range"] = r.into();
        }
        match app.run("insert.chart.fromSheet", p) {
            Ok(_) => return true,
            Err(e) => {
                d.fields.insert("error".into(), e);
                return false;
            }
        }
    }
    cancel
}
