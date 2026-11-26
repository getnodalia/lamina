//! Charts that take their numbers from a spreadsheet range (`.xlsx`, `.csv`, `.tsv`), and the
//! command that reads that range again when the sheet has changed.

use lamina_model::chart::{ChartSource, Series};
use nodalia_sheet::{CellRef, Range, Value as Cell};
use nodalia_xlsx::{Limits, Package, SheetKind};

use super::chart::chart_type;
use super::{Chart, EngineError, Result, Session, Shape, ShapeKind, Value, Xfrm, add_shape, bad, edit_shapes, json, rect_param, str_param};

/// The most rows and columns a chart reads: a chart of more than this is a table.
const MAX_ROWS: usize = 1000;
const MAX_COLS: usize = 64;

/// What the file held inside the range, row by row.
type Table = Vec<Vec<Cell>>;

/// Where a command says the numbers are.
struct Request {
    source: ChartSource,
    /// The file's bytes when the command carried them (`data`), else read from `source.path`.
    data: Option<Vec<u8>>,
    /// The file's name: its extension says `.csv` from `.tsv` when the bytes do not.
    hint: String,
}

fn request(p: &Value, cmd: &str) -> Result<Request> {
    let mut source = ChartSource {
        sheet: str_param(p, "sheet").map(String::from).filter(|s| !s.trim().is_empty()),
        range: str_param(p, "range").map(String::from).filter(|s| !s.trim().is_empty()),
        by_rows: match str_param(p, "series") {
            None | Some("columns") => false,
            Some("rows") => true,
            Some(_) => return Err(bad(cmd, "`series` is columns or rows")),
        },
        first_row: p.get("firstRow").and_then(Value::as_bool),
        first_column: p.get("firstColumn").and_then(Value::as_bool),
        ..Default::default()
    };
    let (data, hint) = if p.get("data").is_some() {
        let (name, bytes) = super::media_bytes(p, cmd)?;
        (Some(bytes), name)
    } else if let Some(path) = str_param(p, "path") {
        source.path = path.to_string();
        (None, path.to_string())
    } else {
        return Err(bad(cmd, "missing `path` or `data`"));
    };
    // A `Sheet2!A1:C9` range names its sheet.
    if let Some((sheet, range)) = source.range.clone().and_then(|r| r.rsplit_once('!').map(|(a, b)| (a.to_string(), b.to_string()))) {
        let sheet = sheet.trim().trim_matches('\'').replace("''", "'");
        if source.sheet.is_none() && !sheet.is_empty() {
            source.sheet = Some(sheet);
        }
        source.range = Some(range.trim().to_string()).filter(|r| !r.is_empty());
    }
    if let Some(r) = &source.range
        && Range::parse(r).is_none()
    {
        return Err(bad(cmd, format!("`{r}` is not a range such as A1:D9")));
    }
    Ok(Request { source, data, hint })
}

/// The table in `bytes`: a workbook's range, or a delimited text file's.
fn read_table(bytes: &[u8], hint: &str, src: &ChartSource) -> Result<Table> {
    let range = src.range.as_deref().and_then(Range::parse);
    if bytes.starts_with(b"PK") || hint.to_ascii_lowercase().ends_with(".xlsb") || nodalia_xlsx::is_xlsb(bytes) {
        read_workbook(bytes, src.sheet.as_deref(), range)
    } else {
        read_delimited(bytes, hint, range)
    }
}

fn read_workbook(bytes: &[u8], sheet: Option<&str>, range: Option<Range>) -> Result<Table> {
    let fail = |e: nodalia_xlsx::Error| EngineError::Other(format!("the workbook can't be read: {e}"));
    let mut package = Package::open(bytes, &Limits::default()).map_err(fail)?;
    let workbook = package.workbook().map_err(fail)?;
    let strings = package.shared_strings().map_err(fail)?;
    let entry = match sheet {
        Some(name) => workbook
            .sheets
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
            .or_else(|| name.parse::<usize>().ok().and_then(|n| n.checked_sub(1)).and_then(|i| workbook.sheets.get(i)))
            .ok_or_else(|| {
                let names: Vec<&str> = workbook.sheets.iter().map(|s| s.name.as_str()).collect();
                EngineError::Other(format!("the workbook has no sheet named `{name}` (it has {})", names.join(", ")))
            })?,
        None => workbook
            .sheets
            .iter()
            .find(|s| s.kind == SheetKind::Worksheet)
            .ok_or_else(|| EngineError::Other("the workbook has no worksheet".into()))?,
    };
    let mut cells: Vec<(CellRef, Cell)> = Vec::new();
    package
        .sheet(entry, |row| {
            for c in &row.cells {
                if range.is_some_and(|r| !r.contains(c.at)) {
                    continue;
                }
                let v = c.value.resolve(&strings, workbook.date_1904);
                if !v.is_empty() && cells.len() < MAX_ROWS * MAX_COLS * 4 {
                    cells.push((c.at, v));
                }
            }
            Ok(())
        })
        .map_err(fail)?;
    Ok(grid(cells, range))
}

fn read_delimited(bytes: &[u8], hint: &str, range: Option<Range>) -> Result<Table> {
    let text = String::from_utf8_lossy(bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let delim = if hint.to_ascii_lowercase().ends_with(".tsv") { b'\t' } else { nodalia_csv::sniff_delimiter(text, '"') };
    let records = nodalia_csv::records(text, delim as char, '"', MAX_ROWS * 4, 1024);
    let mut cells = Vec::new();
    for (r, rec) in records.iter().enumerate() {
        for (c, field) in rec.iter().enumerate() {
            let at = CellRef::new(r as u32, c as u32);
            if range.is_some_and(|rg| !rg.contains(at)) {
                continue;
            }
            let t = field.trim();
            if t.is_empty() {
                continue;
            }
            cells.push((at, number(t).map_or_else(|| Cell::text(t), Cell::number)));
        }
    }
    Ok(grid(cells, range))
}

/// The cells as a rectangle (the range, or what the data spans), capped to what a chart reads.
fn grid(cells: Vec<(CellRef, Cell)>, range: Option<Range>) -> Table {
    let first = range.map_or_else(
        || {
            let r = cells.iter().map(|(c, _)| c.row).min().unwrap_or(0);
            let c = cells.iter().map(|(c, _)| c.col).min().unwrap_or(0);
            CellRef::new(r, c)
        },
        |r| r.first,
    );
    let last = range.map_or_else(
        || CellRef::new(cells.iter().map(|(c, _)| c.row).max().unwrap_or(0), cells.iter().map(|(c, _)| c.col).max().unwrap_or(0)),
        |r| r.last,
    );
    let rows = ((last.row.saturating_sub(first.row)) as usize + 1).min(MAX_ROWS);
    let cols = ((last.col.saturating_sub(first.col)) as usize + 1).min(MAX_COLS);
    let mut table = vec![vec![Cell::Empty; cols]; rows];
    for (at, v) in cells {
        let (r, c) = ((at.row.saturating_sub(first.row)) as usize, (at.col.saturating_sub(first.col)) as usize);
        if let Some(slot) = table.get_mut(r).and_then(|row| row.get_mut(c)) {
            *slot = v;
        }
    }
    table
}

/// A number written the way people type it: `1,250.5`, `12 %`, `-3`.
fn number(t: &str) -> Option<f64> {
    let t = t.trim();
    let (t, percent) = t.strip_suffix('%').map_or((t, false), |x| (x.trim(), true));
    if t.is_empty() || !t.bytes().next().is_some_and(|b| b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.')) {
        return None;
    }
    let plain = if t.contains(',') && !t.contains('.') && t.matches(',').count() == 1 && t.rsplit(',').next().is_some_and(|d| d.len() != 3) {
        t.replace(',', ".")
    } else {
        t.replace(',', "")
    };
    let v: f64 = plain.parse().ok()?;
    v.is_finite().then_some(if percent { v / 100.0 } else { v })
}

fn is_text(v: &Cell) -> bool {
    matches!(v, Cell::Text(_))
}

fn label(v: &Cell) -> String {
    match v {
        Cell::Text(t) => t.to_string(),
        Cell::Number(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", *n as i64),
        Cell::Number(n) => format!("{n}"),
        Cell::Bool(b) => if *b { "TRUE" } else { "FALSE" }.into(),
        Cell::Error(_) | Cell::Empty => String::new(),
    }
}

fn value(v: &Cell) -> Option<f64> {
    match v {
        Cell::Number(n) if n.is_finite() => Some(*n),
        Cell::Bool(b) => Some(f64::from(u8::from(*b))),
        Cell::Text(t) => number(t),
        _ => None,
    }
}

/// Categories and series out of a table. The first row names the series when it is text (or
/// when told so); the first column labels the categories when it is text (or when told so).
fn shape(mut table: Table, src: &ChartSource) -> Result<(Vec<String>, Vec<Series>)> {
    let (mut first_row, mut first_col) = (src.first_row, src.first_column);
    if src.by_rows {
        let cols = table.iter().map(Vec::len).max().unwrap_or(0);
        table = (0..cols).map(|c| table.iter().map(|r| r.get(c).cloned().unwrap_or_default()).collect()).collect();
        std::mem::swap(&mut first_row, &mut first_col);
    }
    if table.iter().all(|r| r.iter().all(Cell::is_empty)) {
        return Err(EngineError::Other("there are no values in that range".into()));
    }
    let width = table.iter().map(Vec::len).max().unwrap_or(0);
    // Labels down the first column: text under the header row.
    let labels = first_col.unwrap_or_else(|| {
        let body = table.iter().skip(1).filter_map(|r| r.first()).filter(|v| !v.is_empty());
        let (texts, all) = body.fold((0, 0), |(t, a), v| (t + usize::from(is_text(v)), a + 1));
        width > 1 && all > 0 && texts * 2 > all
    });
    let skip_col = usize::from(labels);
    // A header row: text across the value columns.
    let header = first_row.unwrap_or_else(|| {
        let cells: Vec<&Cell> = table.first().into_iter().flat_map(|r| r.iter().skip(skip_col)).filter(|v| !v.is_empty()).collect();
        table.len() > 1 && !cells.is_empty() && cells.iter().all(|v| is_text(v) && number_free(v))
    });
    let skip_row = usize::from(header);
    let rows = table.get(skip_row..).unwrap_or_default();
    let categories: Vec<String> =
        rows.iter().enumerate().map(|(i, r)| if labels { r.first().map(label).unwrap_or_default() } else { format!("{}", i + 1) }).collect();
    let series: Vec<Series> = (skip_col..width)
        .map(|c| Series {
            name: if header {
                table.first().and_then(|r| r.get(c)).map(label).filter(|n| !n.is_empty()).unwrap_or_else(|| format!("Series {}", c - skip_col + 1))
            } else {
                format!("Series {}", c - skip_col + 1)
            },
            values: rows.iter().map(|r| r.get(c).and_then(value)).collect(),
            ..Default::default()
        })
        .filter(|s| s.values.iter().any(Option::is_some))
        .collect();
    if series.is_empty() {
        return Err(EngineError::Other("that range holds no numbers to chart".into()));
    }
    Ok((categories, series))
}

fn number_free(v: &Cell) -> bool {
    matches!(v, Cell::Text(t) if number(t).is_none())
}

/// Read the range a request names into categories and series.
fn load(req: &Request, cmd: &str) -> Result<(Vec<String>, Vec<Series>)> {
    let bytes = match &req.data {
        Some(b) => b.clone(),
        None => crate::io::read_file(&req.source.path).map_err(EngineError::Other)?,
    };
    let table = read_table(&bytes, &req.hint, &req.source)?;
    shape(table, &req.source).map_err(|e| match e {
        EngineError::Other(m) => bad(cmd, m),
        e => e,
    })
}

/// Insert › Chart from Spreadsheet: a chart of a range of an `.xlsx`, `.csv` or `.tsv`, linked to it.
pub(super) fn chart_from_sheet(s: &mut Session, p: &Value) -> Result<Value> {
    // No file named: the dialog asks for it.
    if p.get("path").is_none() && p.get("data").is_none() {
        s.ui_requests.push(crate::UiRequest::Dialog { id: "chartFromSheet".into(), params: json!({}) });
        return Ok(json!({"dialog": "chartFromSheet"}));
    }
    let req = request(p, "insert.chart.fromSheet")?;
    let (categories, series) = load(&req, "insert.chart.fromSheet")?;
    let mut c = Chart::sample(chart_type(str_param(p, "type").unwrap_or("column")));
    c.categories = categories;
    c.series = series;
    if let Some(t) = p.get("title") {
        c.title = t.as_str().map(String::from);
    } else {
        c.title = None;
    }
    c.source = Some(req.source);
    let size = s.doc()?.doc.slide_size;
    let rect = rect_param(p, "rect").unwrap_or(Xfrm::new(size.width * 0.15, size.height * 0.15, size.width * 0.7, size.height * 0.7));
    let id = add_shape(s, "insert.chart.fromSheet", Shape { xfrm: Some(rect), kind: ShapeKind::Chart(Box::new(c)), ..Default::default() }, true)?;
    Ok(json!({"id": id}))
}

/// Chart Design › Update from Source: read the chart's range again and keep its looks.
pub(super) fn chart_refresh(s: &mut Session, p: &Value) -> Result<Value> {
    // The chart's recorded source, unless the command names another file.
    let current: Option<ChartSource> = {
        let ids = super::super::targets(s, p)?;
        let st = s.doc()?;
        ids.iter().find_map(|id| match st.shape(*id).map(|sh| &sh.kind) {
            Some(ShapeKind::Chart(c)) => c.source.clone(),
            _ => None,
        })
    };
    let src = match (str_param(p, "path"), current) {
        (Some(path), Some(mut cur)) => {
            cur.path = path.to_string();
            cur
        }
        (None, Some(cur)) if !cur.path.is_empty() => cur,
        (Some(path), None) => ChartSource { path: path.to_string(), ..Default::default() },
        (None, Some(_)) => return Err(bad("chart.refreshFromSource", "this chart's data came with the command, not from a file: give `path`")),
        (None, None) => return Err(bad("chart.refreshFromSource", "this chart has no spreadsheet source: give `path`, `sheet` and `range`")),
    };
    let req = Request { hint: src.path.clone(), source: src.clone(), data: None };
    let (categories, series) = load(&req, "chart.refreshFromSource")?;
    edit_shapes(s, p, "chart.refreshFromSource", |sh| match &mut sh.kind {
        ShapeKind::Chart(c) => {
            c.categories = categories.clone();
            // Series formatting stays with its position.
            let old = std::mem::take(&mut c.series);
            c.series = series
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let mut x = old.get(i).cloned().unwrap_or_default();
                    x.name = n.name.clone();
                    x.values = n.values.clone();
                    x
                })
                .collect();
            c.category_levels.clear();
            c.source = Some(src.clone());
            c.raw = None;
            Ok(())
        }
        _ => Err(bad("chart.refreshFromSource", "not a chart")),
    })?;
    Ok(json!({"series": series.len(), "categories": categories.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src() -> ChartSource {
        ChartSource::default()
    }

    #[test]
    fn numbers_are_read_the_way_people_type_them() {
        assert_eq!(number("1,250.5"), Some(1250.5));
        assert_eq!(number("12,5"), Some(12.5));
        assert_eq!(number("1,250"), Some(1250.0));
        assert_eq!(number("12 %"), Some(0.12));
        assert_eq!(number("-3"), Some(-3.0));
        assert_eq!(number("Q1"), None);
        assert_eq!(number(""), None);
    }

    #[test]
    fn csv_with_a_header_row_and_labels_becomes_series_and_categories() {
        let csv = b"Quarter,Sales,Costs\nQ1,10,4\nQ2,12,5\nQ3,9,6\n";
        let t = read_table(csv, "sales.csv", &src()).expect("table");
        let (cats, series) = shape(t, &src()).expect("shape");
        assert_eq!(cats, ["Q1", "Q2", "Q3"]);
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "Sales");
        assert_eq!(series[0].values, [Some(10.0), Some(12.0), Some(9.0)]);
        assert_eq!(series[1].name, "Costs");
    }

    #[test]
    fn plain_numbers_get_numbered_categories_and_series_names() {
        let t = read_table(b"1,2\n3,4\n5,6\n", "n.csv", &src()).expect("table");
        let (cats, series) = shape(t, &src()).expect("shape");
        assert_eq!(cats, ["1", "2", "3"]);
        assert_eq!(series.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Series 1", "Series 2"]);
    }

    #[test]
    fn rows_can_be_the_series() {
        let csv = b"Region,Jan,Feb\nNorth,1,2\nSouth,3,4\n";
        let source = ChartSource { by_rows: true, ..src() };
        let t = read_table(csv, "r.csv", &source).expect("table");
        let (cats, series) = shape(t, &source).expect("shape");
        assert_eq!(cats, ["Jan", "Feb"]);
        assert_eq!(series.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["North", "South"]);
        assert_eq!(series[1].values, [Some(3.0), Some(4.0)]);
    }

    #[test]
    fn a_range_cuts_the_table_and_a_semicolon_file_is_sniffed() {
        let csv = b"x;y;z\nA;1;100\nB;2;200\nC;3;300\n";
        let source = ChartSource { range: Some("A1:B3".into()), ..src() };
        let t = read_table(csv, "s.csv", &source).expect("table");
        let (cats, series) = shape(t, &source).expect("shape");
        assert_eq!(cats, ["A", "B"]);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].name, "y");
    }

    #[test]
    fn nothing_to_chart_is_an_error() {
        let t = read_table(b"a,b\nc,d\n", "t.csv", &src()).expect("table");
        assert!(shape(t, &src()).is_err());
        let t = read_table(b"", "e.csv", &src()).expect("table");
        assert!(shape(t, &src()).is_err());
    }

    #[test]
    fn hostile_input_is_bounded() {
        // A wide, long file is cut to what a chart reads.
        let row = (0..500).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
        let csv = vec![row; 5000].join("\n");
        let t = read_table(csv.as_bytes(), "big.csv", &src()).expect("table");
        assert!(t.len() <= MAX_ROWS && t.iter().all(|r| r.len() <= MAX_COLS));
        assert!(read_table(b"PK\x03\x04junk", "x.xlsx", &src()).is_err());
    }
}
