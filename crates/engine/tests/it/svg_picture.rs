//! An SVG picture is inserted at its own size, drawn sharp on the slide, and kept by saves.

use lamina_engine::Session;
use lamina_engine::cmd::base64_encode;
use lamina_model::ShapeKind;
use lamina_render::{RenderOpts, render_slide};
use serde_json::json;

/// A 120 × 60 pt-ish drawing: a blue disc on the left, nothing on the right.
const ICON: &str =
    r##"<svg xmlns="http://www.w3.org/2000/svg" width="160" height="80" viewBox="0 0 160 80"><circle cx="40" cy="40" r="36" fill="#1d4ed8"/></svg>"##;

fn session_with_icon() -> (Session, serde_json::Value) {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).expect("new deck");
    let r = s
        .execute("insert.picture", &json!({"name": "icon.svg", "data": base64_encode(ICON.as_bytes()), "rect": [100.0, 100.0, 320.0, 160.0]}))
        .expect("insert svg");
    (s, r["id"].clone())
}

#[test]
fn an_svg_inserts_as_its_vector_with_a_png_fallback() {
    let (s, id) = session_with_icon();
    let doc = s.doc().expect("doc");
    let pic = doc.shapes().iter().find(|sh| json!(sh.id) == id).expect("shape").clone();
    let ShapeKind::Picture { fill } = &pic.kind else { panic!("picture") };
    // As PowerPoint keeps an icon: a PNG every program can show, and the SVG it was drawn from.
    let fallback = doc.doc.media(fill.media).expect("fallback");
    assert_eq!(fallback.content_type, "image/png");
    assert_eq!(fallback.name, "icon.png");
    let vector = doc.doc.media(fill.svg.expect("svg source")).expect("svg media");
    assert_eq!(vector.content_type, "image/svg+xml");
    assert!(vector.data.starts_with(b"<svg"));
}

#[test]
fn an_svg_without_a_rect_is_sized_by_its_own_width_and_height() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).expect("new deck");
    let r = s.execute("insert.picture", &json!({"name": "icon.svg", "data": base64_encode(ICON.as_bytes())})).expect("insert");
    let doc = s.doc().expect("doc");
    let pic = doc.shapes().iter().find(|sh| json!(sh.id) == r["id"]).expect("shape").clone();
    let x = pic.xfrm.expect("xfrm");
    // 160 × 80 px at 96 dpi is 120 × 60 pt, which fits the slide untouched.
    assert!((x.w - 120.0).abs() < 0.5 && (x.h - 60.0).abs() < 0.5, "{x:?}");
}

#[test]
fn an_svg_is_drawn_on_the_slide_and_stays_sharp_when_zoomed() {
    let (s, _) = session_with_icon();
    let doc = s.doc().expect("doc");
    let slide =
        doc.doc.slides.iter().position(|sl| sl.shapes.iter().any(|sh| matches!(sh.kind, ShapeKind::Picture { .. }))).expect("slide with the picture");
    for scale in [1.0, 4.0] {
        let img = render_slide(&doc.doc, slide, &RenderOpts { scale, ..Default::default() });
        // The disc centre is at (100 + 80 * 0.25..): 40/160 of the width, half the height.
        let (cx, cy) = (((100.0 + 320.0 * 0.25) * scale) as u32, ((100.0 + 80.0) * scale) as u32);
        let px = img.pixel(cx, cy);
        assert!(px[2] > 150 && px[0] < 80, "blue at the disc centre (scale {scale}): {px:?}");
        // Its right edge is crisp: well inside the frame but past the disc it is the slide background.
        let (ox, oy) = (((100.0 + 320.0 * 0.85) * scale) as u32, ((100.0 + 80.0) * scale) as u32);
        let outside = img.pixel(ox, oy);
        assert!(outside[2] < 150 || outside[0] > 150, "no disc far right (scale {scale}): {outside:?}");
    }
}

#[test]
fn an_svg_survives_a_pptx_save() {
    let (s, _) = session_with_icon();
    let doc = s.doc().expect("doc").doc.as_ref().clone();
    let back = lamina_pptx::import(&lamina_pptx::export(&doc).expect("export")).expect("import");
    let pic = back.slides.iter().flat_map(|sl| &sl.shapes).find(|sh| matches!(sh.kind, ShapeKind::Picture { .. })).expect("picture after pptx");
    let ShapeKind::Picture { fill } = &pic.kind else { unreachable!() };
    assert_eq!(back.media(fill.media).expect("fallback").content_type, "image/png");
    let vector = back.media(fill.svg.expect("svg source kept")).expect("svg media");
    assert_eq!(vector.content_type, "image/svg+xml");
    assert!(vector.data.starts_with(b"<svg"));

    // The file says it the way PowerPoint does: a PNG blip with the SVG in an extension.
    let zip = lamina_pptx::export(&doc).expect("export");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).expect("zip");
    let mut slide = String::new();
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).expect("entry");
        if f.name().expect("valid zip entry name").starts_with("ppt/slides/slide") && f.name().expect("valid zip entry name").ends_with(".xml") {
            let mut text = String::new();
            std::io::Read::read_to_string(&mut f, &mut text).expect("read");
            if text.contains("svgBlip") {
                slide = text;
            }
        }
    }
    assert!(slide.contains("{96DAC541-7B7A-43D3-8B79-37D633B846F1}"), "the SVG extension is written");
    assert!(slide.contains("asvg:svgBlip"));
}

#[test]
fn a_broken_svg_is_refused_with_a_message() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).expect("new deck");
    let err = s.execute("insert.picture", &json!({"name": "bad.svg", "data": base64_encode(b"<svg><g>")})).expect_err("broken svg");
    assert!(format!("{err}").contains("can't be read"), "{err}");
}
