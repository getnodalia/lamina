//! A sample deck built entirely in code (our own text, shapes and data), for demos, screenshots
//! and tests: `lamina --sample`.

use serde_json::{Value, json};

use crate::{Result, Session};

fn run(s: &mut Session, id: &str, p: Value) -> Result<Value> {
    s.execute(id, &p)
}

fn last_id(v: &Value) -> u64 {
    v.get("id").and_then(Value::as_u64).unwrap_or(0)
}

// NASA astronaut photographs of the aurora from the International Space Station, public domain
// (`assets/sample/`, credited in `ATTRIBUTION.md`), cropped to the slide's 16:9.
static SKY: &[u8] = include_bytes!("../../../assets/sample/aurora-atlantic.jpg");
static ORBIT: &[u8] = include_bytes!("../../../assets/sample/aurora-space.jpg");
static NIGHT: &[u8] = include_bytes!("../../../assets/sample/aurora-midwest.jpg");

/// The current slide's background becomes `photo`.
fn photo_background(s: &mut Session, photo: &[u8]) -> Result<()> {
    run(s, "design.background", json!({"picture": crate::cmd::base64_encode(photo)}))?;
    Ok(())
}

/// A calm, mostly transparent dark panel behind the text, sent to the back of the slide's shapes.
fn veil(s: &mut Session, rect: [f64; 4], transparency: f64) -> Result<()> {
    let v = run(s, "shape.insert", json!({"preset": "rect", "rect": rect}))?;
    let id = last_id(&v);
    run(s, "shape.fill", json!({"id": id, "color": "#050A18", "transparency": transparency}))?;
    run(s, "shape.line", json!({"id": id, "none": true}))?;
    run(s, "shape.rename", json!({"id": id, "name": "Veil"}))?;
    run(s, "edit.select", json!({"ids": [id]}))?;
    run(s, "arrange.sendToBack", json!({}))?;
    Ok(())
}

/// Every shape of the current slide gets `color` as its text colour.
fn paint_text(s: &mut Session, color: &str) -> Result<()> {
    let ids: Vec<u64> = s.doc()?.current_slide().map(|x| x.shapes.iter().map(|s| s.id.0 as u64).collect()).unwrap_or_default();
    run(s, "edit.select", json!({"ids": ids}))?;
    run(s, "format.color", json!({"color": color}))?;
    Ok(())
}

/// Open the sample presentation in a new document.
pub fn open_sample(s: &mut Session) -> Result<()> {
    run(s, "file.new", json!({"theme": "Lámina"}))?;
    let st = s.doc()?;
    let title = st.current_slide().and_then(|x| x.shapes.first()).map(|x| x.id.0).unwrap_or(0);
    let sub = st.current_slide().and_then(|x| x.shapes.get(1)).map(|x| x.id.0).unwrap_or(0);
    run(s, "text.set", json!({"id": title, "text": "Northern Lights"}))?;
    run(s, "text.set", json!({"id": sub, "text": "A short tour of the aurora — where it comes from, when to look, and how to photograph it"}))?;
    run(s, "slide.notes", json!({"text": "Welcome everyone. This deck was made in Lámina."}))?;
    // The aurora seen from orbit, full bleed, with the title in the starry sky above it.
    photo_background(s, SKY)?;
    run(s, "shape.setBounds", json!({"ids": [title], "x": 80, "y": 96, "w": 800, "h": 100}))?;
    run(s, "shape.setBounds", json!({"ids": [sub], "x": 150, "y": 206, "w": 660, "h": 70}))?;
    paint_text(s, "#FFFFFF")?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    // Agenda.
    run(s, "slide.new", json!({"layout": "titleAndContent", "title": "What we'll cover"}))?;
    let body = s.doc()?.current_slide().and_then(|x| x.shapes.get(1)).map(|x| x.id.0).unwrap_or(0);
    run(
        s,
        "text.set",
        json!({"id": body, "text": "The science: solar wind meets the magnetosphere\nWhere and when to see it\n\tHigh latitudes, dark skies, long winter nights\n\tKp index and space-weather forecasts\nPhotographing the aurora\nSafety and etiquette in the field"}),
    )?;
    run(s, "transition.set", json!({"kind": "push", "option": "u"}))?;
    run(s, "edit.select", json!({"ids": [body]}))?;
    run(s, "animation.set", json!({"effect": "fade", "class": "entrance"}))?;
    run(s, "animation.options", json!({"textBuild": "byParagraph"}))?;

    // Section header.
    run(s, "slide.new", json!({"layout": "sectionHeader", "title": "The Science", "body": "Charged particles, magnetic fields and glowing gases"}))?;
    photo_background(s, ORBIT)?;
    veil(s, [0.0, 290.0, 960.0, 130.0], 0.42)?;
    paint_text(s, "#FFFFFF")?;
    run(s, "transition.set", json!({"kind": "wipe", "option": "r"}))?;

    // Process diagram from shapes.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": "From the Sun to your sky"}))?;
    // Two colours only: the steps in the deck's dark blue, the result in aurora green.
    let steps = [("Solar wind", "dk2"), ("Magnetosphere", "dk2"), ("Particles collide", "dk2"), ("Light!", "accent3")];
    let mut prev: Option<u64> = None;
    for (i, (label, color)) in steps.iter().enumerate() {
        let x = 60.0 + i as f64 * 220.0;
        let v = run(s, "shape.insert", json!({"preset": if i == 3 { "ellipse" } else { "roundRect" }, "rect": [x, 210, 190, 120], "text": label}))?;
        let id = last_id(&v);
        run(s, "shape.fill", json!({"id": id, "color": color}))?;
        run(s, "shape.line", json!({"id": id, "none": true}))?;
        run(s, "edit.select", json!({"ids": [id]}))?;
        run(s, "format.size", json!({"size": 17}))?;
        run(s, "format.bold", json!({"on": true}))?;
        run(s, "format.color", json!({"color": "#FFFFFF"}))?;
        run(s, "animation.add", json!({"effect": "zoom", "class": "entrance", "start": if i == 0 { "onClick" } else { "afterPrevious" }}))?;
        if let Some(p) = prev {
            let _ = p;
            let a = run(s, "shape.insert", json!({"preset": "rightArrow", "rect": [x - 30.0, 262, 20, 16]}))?;
            run(s, "shape.fill", json!({"id": last_id(&a), "color": "tx2"}))?;
            run(s, "shape.line", json!({"id": last_id(&a), "none": true}))?;
        }
        prev = Some(id);
    }
    let cap =
        run(s, "insert.textBox", json!({"rect": [60, 380, 840, 60], "text": "Oxygen glows green and red; nitrogen adds blue and purple fringes."}))?;
    run(s, "edit.select", json!({"ids": [last_id(&cap)]}))?;
    run(s, "format.alignCenter", json!({}))?;
    run(s, "format.size", json!({"size": 20}))?;
    run(s, "format.italic", json!({"on": true}))?;
    run(s, "transition.set", json!({"kind": "morph"}))?;

    // A photograph: the same aurora, seen over the lights of a continent.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": "Seen from the Space Station"}))?;
    run(s, "insert.picture", json!({"data": crate::cmd::base64_encode(NIGHT), "name": "Aurora over the Midwest", "rect": [60, 140, 560, 315]}))?;
    let note = run(
        s,
        "insert.textBox",
        json!({"rect": [660, 150, 250, 230], "text": "The Midwest at night, 29 September 2011. A green arc of aurora follows the curve of the Earth, above the city lights of the Great Lakes."}),
    )?;
    run(s, "edit.select", json!({"ids": [last_id(&note)]}))?;
    run(s, "format.size", json!({"size": 18}))?;
    let credit = run(s, "insert.textBox", json!({"rect": [660, 400, 250, 40], "text": "Photo: NASA Earth Observatory"}))?;
    run(s, "edit.select", json!({"ids": [last_id(&credit)]}))?;
    run(s, "format.size", json!({"size": 12}))?;
    run(s, "format.italic", json!({"on": true}))?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    // Chart.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": "Best months to look (clear nights)"}))?;
    run(
        s,
        "insert.chart",
        json!({"type": "column", "rect": [80, 140, 800, 360], "title": "Average clear, dark nights per month", "categories": ["Sep", "Oct", "Nov", "Dec", "Jan", "Feb", "Mar"], "series": [{"name": "Tromsø", "values": [6, 8, 7, 6, 7, 9, 11]}, {"name": "Fairbanks", "values": [9, 10, 8, 9, 11, 12, 13]}, {"name": "Yellowknife", "values": [8, 11, 9, 10, 12, 13, 14]}]}),
    )?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    // Table.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": "Camera settings that work"}))?;
    run(
        s,
        "insert.table",
        json!({"rows": 5, "cols": 3, "rect": [100, 150, 760, 260], "data": [["Setting", "Faint aurora", "Bright, active aurora"], ["Aperture", "f/1.4 – f/2.8", "f/2.8"], ["Shutter", "8 – 15 s", "1 – 4 s"], ["ISO", "1600 – 3200", "800 – 1600"], ["Focus", "Manual, on a bright star", "Manual, on a bright star"]]}),
    )?;
    run(s, "transition.set", json!({"kind": "cover", "option": "l"}))?;

    // Two content: comparison with shapes.
    run(s, "slide.new", json!({"layout": "twoContent", "title": "Field checklist"}))?;
    let st = s.doc()?;
    let (l, r) =
        st.current_slide().map(|x| (x.shapes.get(1).map(|s| s.id.0).unwrap_or(0), x.shapes.get(2).map(|s| s.id.0).unwrap_or(0))).unwrap_or((0, 0));
    run(s, "text.set", json!({"id": l, "text": "Bring\n\tTripod and spare batteries\n\tHeadlamp with a red mode\n\tWarm layers and a thermos"}))?;
    run(
        s,
        "text.set",
        json!({"id": r, "text": "Remember\n\tCheck the forecast and cloud cover\n\tLet your eyes adapt for 20 minutes\n\tLeave no trace"}),
    )?;
    run(s, "transition.set", json!({"kind": "split"}))?;

    // Closing.
    run(s, "slide.new", json!({"layout": "title", "title": "Clear skies!", "body": "Made with Lámina — free and open source"}))?;
    photo_background(s, SKY)?;
    paint_text(s, "#FFFFFF")?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    run(s, "section.add", json!({"name": "Introduction", "at": 0}))?;
    run(s, "section.add", json!({"name": "Science", "at": 2}))?;
    run(s, "section.add", json!({"name": "In the Field", "at": 6}))?;
    run(s, "slide.go", json!({"index": 0}))?;
    run(s, "edit.deselect", json!({}))?;
    run(s, "edit.deselect", json!({}))?;
    // The sample opens clean: no undo history, not dirty.
    if let Some(st) = s.active_mut() {
        st.history.undo.clear();
        st.history.redo.clear();
        st.saved_doc = st.doc.clone();
        st.title = "Northern Lights".into();
    }
    Ok(())
}
