use super::{Color32, Rect, SlideApp, Tokens, Ui, pos2, theme};

/// About Lámina, as every Nodalia app draws it (`nodalia_theme::about`): the icon, the name and
/// version, one line on what it is, "Part of Nodalia", the links and the licence. Upstream authors
/// are credited in NOTICE, which the last line points to.
pub fn about(_app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.set_width(nodalia_theme::about::measures::WIDTH);
    let look = nodalia_theme::creator::Look::new(theme::palette(t.dark)).strong_family(theme::bold(1.0).family);
    let version = version_line();
    let links = [
        nodalia_theme::about::Link { label: crate::i18n::t("getnodalia.com/lamina"), url: lamina_engine::links::APP_PAGE },
        nodalia_theme::about::Link { label: crate::i18n::t("Source code"), url: lamina_engine::links::GITHUB },
        nodalia_theme::about::Link { label: crate::i18n::t("Report a problem or suggest an idea"), url: lamina_engine::links::ISSUES },
        nodalia_theme::about::Link { label: crate::i18n::t("Licence: MIT OR Apache-2.0"), url: LICENCE },
    ];
    let info = nodalia_theme::about::Info {
        name: crate::i18n::t("Lámina"),
        version: &version,
        tagline: crate::i18n::t("Presentations and slide shows, in pure Rust."),
        suite: crate::i18n::t("Part of Nodalia"),
        links: &links,
        footer: crate::i18n::t("Includes work by other open-source authors (see NOTICE)."),
    };
    let page = nodalia_theme::about::page(ui, &look, &info, |ui, rect| paint_logo(ui.painter(), rect));
    if let Some(i) = page.picked
        && let Some(link) = links.get(i)
    {
        ui.ctx().open_url(egui::OpenUrl::new_tab(link.url));
    }
}

/// The licence the About window links to.
const LICENCE: &str = "https://github.com/getnodalia/lamina/blob/main/NOTICE";

/// "Version X.Y.Z" plus, for release builds, the short commit and build date. The version comes
/// from `[workspace.package] version` (the single source of truth, `cargo xtask version`); the
/// release workflow sets `LAMINA_BUILD_SHA` / `LAMINA_BUILD_DATE` at build time.
pub fn version_line() -> String {
    let mut s = crate::i18n::tf("Version {version}", &[("version", &env!("CARGO_PKG_VERSION"))]);
    let sha = option_env!("LAMINA_BUILD_SHA").map(|s| s.get(..9).unwrap_or(s)).filter(|s| !s.is_empty());
    let date = option_env!("LAMINA_BUILD_DATE").filter(|s| !s.is_empty());
    match (sha, date) {
        (Some(sha), Some(date)) => s.push_str(&format!(" ({sha}, {date})")),
        (Some(x), None) | (None, Some(x)) => s.push_str(&format!(" ({x})")),
        (None, None) => {}
    }
    s
}

/// Lámina's app icon, the same picture the desktop shows for it.
const ICON_PNG: &[u8] = include_bytes!("../../../../assets/app-icon/hicolor/256x256/apps/dev.danielmiguelt.Lamina.png");

/// Paints Lámina's app icon into `r`. The texture is decoded once per context; if the picture
/// can't be decoded the space stays empty rather than failing the dialog.
pub fn paint_logo(p: &egui::Painter, r: Rect) {
    let ctx = p.ctx();
    let id = egui::Id::unique("lamina_app_icon");
    let texture = ctx.data(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let t = lamina_render::decode_image(ICON_PNG).map(|img| {
            let size = [img.width() as usize, img.height() as usize];
            ctx.load_texture("lamina_app_icon", egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR)
        });
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    if let Some(t) = texture {
        p.image(t.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
}

#[cfg(test)]
mod about_tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    use crate::dialogs::Dialog;
    use crate::{Services, SlideApp};

    /// About is one page in the office apps' layout: no credits tabs, no Cancel/Close row (the
    /// round close button at the top is the way out), and upstream authors only through NOTICE.
    #[test]
    fn about_is_one_page_that_closes_from_its_own_button() {
        let mut app = SlideApp::new(lamina_engine::Session::with_new(), Services::default());
        app.dialog = Some(Dialog::new("about"));
        let mut h = Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut SlideApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
        h.run_steps(3);
        // The shared page paints its text; its links are real widgets.
        assert!(h.query_by_label("Source code").is_some());
        for gone in ["Contributors", "Models", "Cancel"] {
            assert!(h.query_by_label(gone).is_none(), "{gone} is not in About");
        }
        h.get_by_label("Close window").click();
        h.run_steps(2);
        assert!(h.state().dialog.is_none(), "the close button closes About");
    }
}
