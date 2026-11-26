//! A chart takes its numbers from a spreadsheet range, remembers where they came from, and reads
//! them again on request without losing how the chart looks.

use lamina_engine::Session;
use lamina_engine::cmd::base64_encode;
use lamina_model::ShapeKind;
use serde_json::{Value, json};

/// A workbook with a `Sales` sheet (header row, quarter labels, two series) and a `Notes` sheet.
fn workbook(q2: f64) -> Vec<u8> {
    let sales = format!(
        r#"<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c></row>
<row r="2"><c r="A2" t="s"><v>3</v></c><c r="B2"><v>10</v></c><c r="C2"><v>4</v></c></row>
<row r="3"><c r="A3" t="s"><v>4</v></c><c r="B3"><v>{q2}</v></c><c r="C3"><v>5</v></c></row>
<row r="4"><c r="A4" t="s"><v>5</v></c><c r="B4"><v>9</v></c><c r="C4"><v>6</v></c></row>"#
    );
    let notes = r#"<row r="1"><c r="A1" t="s"><v>6</v></c></row>"#;
    nodalia_xlsx::testing::workbook(&[("Sales", &sales), ("Notes", notes)], &["Quarter", "Revenue", "Costs", "Q1", "Q2", "Q3", "Only text"])
        .expect("workbook")
}

fn deck() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).expect("new deck");
    s
}

fn chart_of(s: &Session, id: &Value) -> lamina_model::Chart {
    let doc = s.doc().expect("doc");
    let sh = doc.shapes().iter().find(|sh| &json!(sh.id) == id).expect("shape").clone();
    let ShapeKind::Chart(c) = sh.kind else { panic!("a chart") };
    *c
}

fn insert(s: &mut Session, extra: Value) -> Value {
    let mut p = json!({"name": "book.xlsx", "data": base64_encode(&workbook(12.0))});
    if let (Some(o), Some(e)) = (p.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    s.execute("insert.chart.fromSheet", &p).expect("chart")["id"].clone()
}

#[test]
fn a_chart_reads_series_and_categories_from_a_workbook_sheet() {
    let mut s = deck();
    let id = insert(&mut s, json!({"sheet": "Sales"}));
    let c = chart_of(&s, &id);
    assert_eq!(c.categories, ["Q1", "Q2", "Q3"]);
    assert_eq!(c.series.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["Revenue", "Costs"]);
    assert_eq!(c.series[0].values, [Some(10.0), Some(12.0), Some(9.0)]);
    assert_eq!(c.series[1].values, [Some(4.0), Some(5.0), Some(6.0)]);
    let src = c.source.expect("remembers its source");
    assert_eq!(src.sheet.as_deref(), Some("Sales"));
}

#[test]
fn the_first_sheet_is_the_default_and_a_range_can_name_its_sheet() {
    let mut s = deck();
    let id = insert(&mut s, json!({"range": "Sales!A1:B4"}));
    let c = chart_of(&s, &id);
    assert_eq!(c.series.len(), 1);
    assert_eq!(c.series[0].name, "Revenue");
    assert_eq!(c.source.expect("source").sheet.as_deref(), Some("Sales"));
}

#[test]
fn rows_can_be_the_series() {
    let mut s = deck();
    let id = insert(&mut s, json!({"sheet": "Sales", "series": "rows", "range": "A1:C3"}));
    let c = chart_of(&s, &id);
    assert_eq!(c.categories, ["Revenue", "Costs"]);
    assert_eq!(c.series.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["Q1", "Q2"]);
}

#[test]
fn a_csv_inserts_the_same_way() {
    let mut s = deck();
    let csv = "Month;Visits\nJan;120\nFeb;150\nMar;90\n";
    let r = s.execute("insert.chart.fromSheet", &json!({"name": "v.csv", "data": base64_encode(csv.as_bytes()), "type": "line"})).expect("csv chart");
    let c = chart_of(&s, &r["id"]);
    assert_eq!(c.categories, ["Jan", "Feb", "Mar"]);
    assert_eq!(c.series[0].name, "Visits");
    assert_eq!(c.series[0].values[1], Some(150.0));
}

#[test]
fn bad_input_is_an_error_not_a_crash() {
    let mut s = deck();
    let before = s.doc().expect("doc").shapes().len();
    let book = base64_encode(&workbook(1.0));
    for p in [
        json!({"name": "b.xlsx", "data": book, "sheet": "Nope"}),
        json!({"name": "b.xlsx", "data": book, "range": "not a range"}),
        json!({"name": "b.xlsx", "data": book, "sheet": "Notes"}),
        json!({"name": "b.xlsx", "data": book, "series": "diagonal"}),
        json!({"name": "b.xlsx", "data": base64_encode(b"PK\x03\x04 not a workbook")}),
        json!({"name": "e.csv", "data": ""}),
    ] {
        assert!(s.execute("insert.chart.fromSheet", &p).is_err(), "{p}");
    }
    assert_eq!(s.doc().expect("doc").shapes().len(), before, "nothing was inserted");
    // No file at all asks the UI for its dialog instead of failing.
    let r = s.execute("insert.chart.fromSheet", &json!({})).expect("dialog");
    assert_eq!(r["dialog"], "chartFromSheet");
}

#[test]
fn update_from_source_reads_the_file_again_and_keeps_the_formatting() {
    let dir = std::env::temp_dir().join(format!("lamina_sheet_chart_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    let path = dir.join("book.xlsx");
    std::fs::write(&path, workbook(12.0)).expect("write");
    let path = path.to_string_lossy().to_string();

    let mut s = deck();
    let r = s.execute("insert.chart.fromSheet", &json!({"path": path, "sheet": "Sales"})).expect("chart");
    let id = r["id"].clone();
    s.execute("chart.options", &json!({"title": "Results", "dataLabels": true, "ids": [id]})).expect("style");
    assert_eq!(chart_of(&s, &id).series[0].values[1], Some(12.0));

    // The sheet changes on disk; the chart follows when asked, not before.
    std::fs::write(&path, workbook(30.0)).expect("rewrite");
    assert_eq!(chart_of(&s, &id).series[0].values[1], Some(12.0));
    s.execute("chart.refreshFromSource", &json!({"ids": [id]})).expect("refresh");
    let c = chart_of(&s, &id);
    assert_eq!(c.series[0].values[1], Some(30.0));
    assert_eq!(c.title.as_deref(), Some("Results"));
    assert!(c.data_labels);

    // One undo step.
    s.execute("edit.undo", &json!({})).expect("undo");
    assert_eq!(chart_of(&s, &id).series[0].values[1], Some(12.0));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_chart_without_a_file_cannot_refresh_and_says_why() {
    let mut s = deck();
    let id = insert(&mut s, json!({"sheet": "Sales"}));
    let e = s.execute("chart.refreshFromSource", &json!({"ids": [id]})).expect_err("no file");
    assert!(e.to_string().contains("path"), "{e}");
    let plain = s.execute("insert.chart", &json!({})).expect("plain chart")["id"].clone();
    assert!(s.execute("chart.refreshFromSource", &json!({"ids": [plain]})).is_err());
}

#[test]
fn the_source_survives_a_native_save() {
    let mut s = deck();
    let id = insert(&mut s, json!({"sheet": "Sales", "range": "A1:C4"}));
    let dir = std::env::temp_dir().join(format!("lamina_sheet_chart_save_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    let file = dir.join("deck.lamina").to_string_lossy().to_string();
    s.execute("file.saveAs", &json!({"path": file})).expect("save");
    let mut t = Session::new();
    t.execute("file.open", &json!({"path": file})).expect("open");
    let doc = t.doc().expect("doc");
    let sh = doc.shapes().iter().find(|sh| matches!(sh.kind, ShapeKind::Chart(_))).expect("chart").clone();
    let ShapeKind::Chart(c) = sh.kind else { unreachable!() };
    assert_eq!(c.source.expect("source").range.as_deref(), Some("A1:C4"));
    assert_eq!(c.series.len(), chart_of(&s, &id).series.len());
    std::fs::remove_dir_all(&dir).ok();
}
