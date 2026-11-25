//! SVG pictures (`image/svg+xml`): parsed by `nodalia-svg` (bounded like every hostile file: size,
//! nesting, entities, embedded images) and drawn by resvg into a bitmap of the size the slide
//! needs, so they go through the same crop, tile and recolour path as any other picture.
//!
//! Text in an SVG is drawn only when the file turns it into paths; the picture has no font of
//! its own and the renderer does not borrow one, so a label set in a missing face never shows as
//! a stand-in.

use nodalia_svg::tree::{self, Limits, draw::tiny_skia, usvg};

/// Longest side drawn.
const MAX_SIDE: u32 = 8192;
/// Bytes of pixel buffers a single drawing may keep alive at once.
const BUDGET: u64 = 256 << 20;
/// Stack of the thread that parses and draws: usvg recurses about 15 kB for each nesting level.
#[cfg(not(target_arch = "wasm32"))]
const STACK: usize = 32 << 20;

/// Is `bytes` an SVG or an SVGZ (gzip-compressed SVG)?
pub fn is_svg(bytes: &[u8]) -> bool {
    nodalia_svg::sniff::is_svg(bytes)
        || nodalia_svg::sniff::is_svgz(bytes) && nodalia_svg::svgz::inflate(bytes, 1 << 20).is_ok_and(|b| nodalia_svg::sniff::is_svg(&b))
}

/// Limits sized for the stack this build parses on.
fn limits() -> Limits {
    let mut l = Limits::default();
    if cfg!(target_arch = "wasm32") {
        // A browser's stack is small: about 34 levels fit one megabyte.
        l.tree_depth = 24;
        l.depth = 64;
        l.embedded_depth = 0;
    } else {
        l.tree_depth = 128;
        l.depth = 128;
        l.embedded_depth = 1;
        l.bytes = 16 << 20;
        l.embedded_bytes = 8 << 20;
    }
    l
}

/// Run `f` on a thread with room for usvg's recursion (the calling thread, in a browser).
fn on_big_stack<R: Send>(f: impl FnOnce() -> Option<R> + Send) -> Option<R> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::scope(|s| std::thread::Builder::new().stack_size(STACK).spawn_scoped(s, f).ok()?.join().ok().flatten())
    }
    #[cfg(target_arch = "wasm32")]
    {
        f()
    }
}

fn parse(bytes: &[u8]) -> Option<usvg::Tree> {
    tree::parse(bytes, &limits(), &usvg::Options::default()).ok()
}

/// The drawing's own size in 96-dpi pixels, from its `width` and `height` (or its `viewBox`).
pub fn natural_size(bytes: &[u8]) -> Option<(f64, f64)> {
    on_big_stack(|| {
        let size = parse(bytes)?.size();
        let (w, h) = (size.width() as f64, size.height() as f64);
        (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0).then_some((w, h))
    })
}

/// Draw the SVG `bytes` into straight RGBA with its longer side `side` px (transparent where it
/// draws nothing). `None` when it doesn't parse, has no size, or is too heavy to draw.
pub fn rasterize(bytes: &[u8], side: u32) -> Option<image::RgbaImage> {
    on_big_stack(|| {
        let tree = parse(bytes)?;
        let size = tree.size();
        let (sw, sh) = (size.width() as f64, size.height() as f64);
        if !(sw.is_finite() && sh.is_finite() && sw > 0.0 && sh > 0.0) {
            return None;
        }
        let k = side.clamp(16, MAX_SIDE) as f64 / sw.max(sh);
        let w = (sw * k).round().clamp(1.0, MAX_SIDE as f64) as u32;
        let h = (sh * k).round().clamp(1.0, MAX_SIDE as f64) as u32;
        let ts = tiny_skia::Transform::from_scale((w as f32) / size.width(), (h as f32) / size.height());
        let pm = tree::draw::render(&tree, ts, w, h, 0, BUDGET).ok()?;
        let mut rgba = Vec::with_capacity(pm.data().len());
        for p in pm.pixels() {
            let c = p.demultiply();
            rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
        }
        image::RgbaImage::from_raw(pm.width(), pm.height(), rgba)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED_SQUARE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" viewBox="0 0 40 20"><rect x="0" y="0" width="20" height="20" fill="#ff0000"/></svg>"##;

    #[test]
    fn an_svg_is_recognised_and_other_bytes_are_not() {
        assert!(is_svg(RED_SQUARE.as_bytes()));
        assert!(is_svg(b"<?xml version=\"1.0\"?>\n<svg/>"));
        assert!(!is_svg(b"\x89PNG\r\n\x1a\n"));
        assert!(!is_svg(b"plain text"));
        assert!(!is_svg(&[]));
    }

    #[test]
    fn a_gzipped_svg_is_recognised() {
        let z = nodalia_svg::svgz::compress(RED_SQUARE.as_bytes());
        assert!(is_svg(&z));
        assert!(!is_svg(&nodalia_svg::svgz::compress(b"not a drawing")));
    }

    #[test]
    fn the_natural_size_is_the_drawings_own() {
        assert_eq!(natural_size(RED_SQUARE.as_bytes()), Some((40.0, 20.0)));
        assert_eq!(natural_size(b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 30'/>"), Some((10.0, 30.0)));
    }

    #[test]
    fn it_draws_at_the_asked_size_and_stays_vector_sharp() {
        let small = rasterize(RED_SQUARE.as_bytes(), 40).expect("small");
        let big = rasterize(RED_SQUARE.as_bytes(), 400).expect("big");
        assert_eq!(small.dimensions(), (40, 20));
        assert_eq!(big.dimensions(), (400, 200));
        // The red square covers the left half, with a hard edge at every size and nothing right of it.
        for (img, scale) in [(&small, 1u32), (&big, 10)] {
            assert_eq!(img.get_pixel(5 * scale, 10 * scale).0, [255, 0, 0, 255]);
            assert_eq!(img.get_pixel(30 * scale, 10 * scale).0[3], 0);
            assert_eq!(img.get_pixel(19 * scale + scale / 2, 10 * scale).0, [255, 0, 0, 255]);
            assert_eq!(img.get_pixel(20 * scale + scale / 2, 10 * scale).0[3], 0);
        }
    }

    #[test]
    fn partial_transparency_is_kept_straight() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#0000ff" fill-opacity="0.5"/></svg>"##;
        let img = rasterize(svg.as_bytes(), 10).expect("drawn");
        let p = img.get_pixel(5, 5).0;
        assert!(p[2] >= 250 && p[0] <= 2 && p[1] <= 2, "colour stays blue, not darkened: {p:?}");
        assert!((120..=135).contains(&p[3]), "alpha is half: {p:?}");
    }

    #[test]
    fn hostile_or_broken_files_are_refused_not_crashed() {
        assert!(rasterize(b"<svg", 64).is_none());
        assert!(rasterize(b"", 64).is_none());
        assert!(natural_size(b"<svg xmlns='http://www.w3.org/2000/svg' width='0' height='0'/>").is_none());
        // A billion-laughs entity chain.
        let lol = "<?xml version=\"1.0\"?><!DOCTYPE svg [<!ENTITY a \"aaaaaaaaaa\"><!ENTITY b \"&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;\"><!ENTITY c \"&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;\"><!ENTITY d \"&c;&c;&c;&c;&c;&c;&c;&c;&c;&c;\"><!ENTITY e \"&d;&d;&d;&d;&d;&d;&d;&d;&d;&d;\"><!ENTITY f \"&e;&e;&e;&e;&e;&e;&e;&e;&e;&e;\"><!ENTITY g \"&f;&f;&f;&f;&f;&f;&f;&f;&f;&f;\"><!ENTITY h \"&g;&g;&g;&g;&g;&g;&g;&g;&g;&g;\">]><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"><text>&h;&h;&h;&h;&h;&h;&h;&h;&h;&h;</text></svg>";
        assert!(rasterize(lol.as_bytes(), 64).is_none());
        // Elements nested far past the limit.
        let deep = format!("<svg xmlns='http://www.w3.org/2000/svg' width='10' height='10'>{}{}</svg>", "<g>".repeat(5000), "</g>".repeat(5000));
        assert!(rasterize(deep.as_bytes(), 64).is_none());
    }
}
