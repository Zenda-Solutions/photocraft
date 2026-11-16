//! Zenda patches for Work. Events carry metadata only; Work owns accounts and Theo.
use serde_json::{Value, json};

use crate::PhotocraftApp;

pub const ON: bool = cfg!(feature = "zenda");

/// Events kept for Work when it does not drain them; older ones are dropped first.
const OUTBOX_MAX: usize = 32;

/// Work's orange (#EA6C1E).
pub const ORANGE: egui::Color32 = egui::Color32::from_rgb(234, 108, 30);

/// Width the Theo button takes at the far end of the title bar.
pub const BUTTON_WIDTH: f32 = 74.0;

/// File › Share… in the Zenda build: Work sends the file, the app only says which one.
pub const SHARE_COMMAND: &str = "zenda.share";

/// Help items the Zenda build drops: community links and upstream's update check.
pub const STRIPPED_COMMANDS: &[&str] = &["help.discord", "help.website", "help.artcraftWebsite", "help.checkForUpdates"];

/// The fork, where the source and Zenda's modifications live.
pub const FORK: &str = "https://github.com/Zenda-Solutions/photocraft";

/// What the person asked of Work, oldest first, until Work drains it (`work.events`).
#[derive(Default, Debug)]
pub struct Outbox {
    events: Vec<Value>,
    seq: u64,
}

impl Outbox {
    /// Queue `event` (`work.theo`, `work.share`) with its params.
    pub fn push(&mut self, event: &str, params: Value) {
        self.seq = self.seq.saturating_add(1);
        if self.events.len() == OUTBOX_MAX {
            self.events.remove(0);
        }
        self.events.push(json!({"seq": self.seq, "event": event, "params": params}));
    }

    /// Every queued event, oldest first; the outbox is empty afterwards.
    pub fn drain(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Where help was requested, without image content.
pub fn theo_context(app: &PhotocraftApp, from: &str) -> Value {
    json!({"from": from, "hasDocument": app.session.active().is_some()})
}

/// The active image: its saved path (none before the first save), name and unsaved state.
pub fn share_params(app: &PhotocraftApp) -> Value {
    let doc = app.session.active();
    json!({
        "path": doc.and_then(|d| d.path.as_deref()),
        "dirty": doc.is_some_and(|d| d.is_dirty()),
        "name": doc.map(|d| d.doc.name.as_str()),
    })
}

/// Work's mascot texture, decoded once per context.
fn mascot(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let key = egui::Id::unique("zenda.theo.texture");
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return Some(texture);
    }
    let decoded = photocraft_codecs::decode(include_bytes!("../../../assets/zenda/theo-icon-64.png")).ok()?;
    let rgba = decoded.convert(photocraft_codecs::ChannelLayout::Rgba, photocraft_codecs::SampleType::U8);
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    if rgba.data().len() != w * h * 4 {
        return None;
    }
    let texture = ctx.load_texture("Theo mascot", egui::ColorImage::from_rgba_unmultiplied([w, h], rgba.data()), egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(key, texture.clone()));
    Some(texture)
}

/// Work's mascot and orange brand button, shared by the curated app patches.
pub fn theo_button(ui: &mut egui::Ui, label: &str, hint: &str) -> egui::Response {
    let text = egui::RichText::new("Theo").color(egui::Color32::BLACK).size(11.5);
    let button = match mascot(ui.ctx()) {
        Some(texture) => egui::Button::image_and_text(egui::Image::new((texture.id(), egui::vec2(18.0, 18.0))), text),
        None => egui::Button::new(text),
    };
    let response = ui.add(button.fill(ORANGE).corner_radius(11.0).min_size(egui::vec2(BUTTON_WIDTH, 24.0)));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    response.on_hover_text(hint)
}

/// The Theo button at the far end of the title bar; a press queues `work.theo`.
pub fn title_bar_theo(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    if theo_button(ui, tl!("Ask Theo for help"), tl!("Ask Theo about what is on screen")).clicked() {
        let context = theo_context(app, "button");
        app.zenda.push("work.theo", context);
    }
}

/// Commands the Zenda build takes before upstream's: Share goes to Work, and the dropped Help
/// links open nothing. `None` lets the command run as usual.
pub fn intercept(app: &mut PhotocraftApp, id: &str) -> Option<Value> {
    if !ON {
        return None;
    }
    if id == SHARE_COMMAND {
        let params = share_params(app);
        app.zenda.push("work.share", params.clone());
        app.ui.status = tl!("Work will share this image.").to_owned();
        return Some(params);
    }
    if STRIPPED_COMMANDS.contains(&id) {
        return Some(Value::Null);
    }
    None
}

/// The Zenda build's menus: no community links or update check, and File › Share… after Export.
pub fn adjust_menu(app: &PhotocraftApp, items: &mut Vec<crate::menus::MenuItem>) {
    if !ON {
        return;
    }
    items.retain(|i| !STRIPPED_COMMANDS.contains(&i.id.as_str()));
    let share = crate::menus::MenuItem {
        id: SHARE_COMMAND.into(),
        label: "Share…".into(),
        path: vec!["File".into()],
        shortcut: None,
        enabled: app.session.active().is_some(),
        checked: None,
        color: None,
    };
    let at = items
        .iter()
        .position(|i| i.path.first().map(String::as_str) == Some("File") && i.path.len() >= 2 && i.path[1] == "Export")
        .and_then(|first| items.iter().skip(first).position(|i| !(i.path.len() >= 2 && i.path[1] == "Export")).map(|n| first + n))
        .or_else(|| items.iter().rposition(|i| i.path.first().map(String::as_str) == Some("File")).map(|p| p + 1))
        .unwrap_or(items.len());
    items.insert(at, share);
}

/// The Help links the Zenda build keeps point at the fork.
pub fn link(id: &str, upstream: &'static str) -> Option<&'static str> {
    if !ON {
        return Some(upstream);
    }
    match id {
        "help.github" => Some(FORK),
        "help.reportIssue" => Some("https://github.com/Zenda-Solutions/photocraft/issues"),
        _ if STRIPPED_COMMANDS.contains(&id) => None,
        _ => Some(upstream),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ControlRequest, Outcome};

    fn app() -> PhotocraftApp {
        PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default())
    }

    #[test]
    fn zenda_events_drain_in_order_and_are_bounded() {
        let mut out = Outbox::default();
        for _ in 0..40 {
            out.push("work.theo", json!({"from": "button"}));
        }
        let got = out.drain();
        assert_eq!(got.len(), 32);
        assert_eq!(got[0]["seq"], 9);
        assert!(out.is_empty());
    }

    #[test]
    fn zenda_only_the_zenda_build_exposes_work_events() {
        let mut app = app();
        let context = theo_context(&app, "button");
        app.zenda.push("work.theo", context);
        let (req, _pending) = ControlRequest::new("work.events", json!({}));
        let Outcome::Done(reply) = crate::control::handle(&mut app, &egui::Context::default(), &req) else { panic!("immediate reply") };
        assert_eq!(reply["ok"], ON, "{reply}");
        if ON {
            assert_eq!(reply["result"]["events"][0]["event"], "work.theo");
            assert!(app.zenda.is_empty());
        }
    }

    #[test]
    fn zenda_share_goes_to_work_only_in_the_zenda_build() {
        let mut app = app();
        app.run("file.new", json!({"width": 32, "height": 32})).unwrap();
        let shared = crate::menus::invoke(&mut app, &egui::Context::default(), SHARE_COMMAND, json!({}));
        if ON {
            assert!(shared.is_ok(), "{shared:?}");
            let got = app.zenda.drain();
            assert_eq!(got.len(), 1);
            assert_eq!(got[0]["event"], "work.share");
            assert!(got[0]["params"].get("dirty").is_some());
            assert!(got[0]["params"]["path"].is_null(), "never saved");
        } else {
            assert!(app.zenda.is_empty());
        }
    }

    #[test]
    fn zenda_menus_drop_community_links_and_add_share() {
        let mut app = app();
        app.run("file.new", json!({"width": 32, "height": 32})).unwrap();
        let items = crate::menus::menu_items(&app);
        let has = |id: &str| items.iter().any(|i| i.id == id);
        for id in STRIPPED_COMMANDS {
            assert_eq!(has(id), !ON, "{id}");
        }
        assert_eq!(has(SHARE_COMMAND), ON);
        if ON {
            let share = items.iter().find(|i| i.id == SHARE_COMMAND).unwrap();
            assert!(share.enabled && share.path == ["File"]);
            assert_eq!(crate::links::url_for("help.github"), Some(FORK));
            assert_eq!(crate::links::url_for("help.discord"), None);
        }
    }

    #[test]
    fn zenda_theo_button_has_localized_accessible_names() {
        use egui_kittest::kittest::Queryable;
        let mut cases = vec![("en", "Ask Theo for help")];
        // The translated names need the catalogs embedded (CRAFT_I18N_DIR with Zenda's rows).
        if crate::i18n::Lang::from_code("es").is_some_and(|es| crate::i18n::with_language(es, || tl!("Ask Theo for help") != "Ask Theo for help")) {
            cases.extend([("es", "Pedile ayuda a Theo"), ("pt-br", "Peça ajuda ao Theo")]);
        } else if std::env::var_os("CRAFT_I18N_DIR").is_some() {
            panic!("CRAFT_I18N_DIR is set but has no Spanish name for the Theo button: run zenda/i18n.ps1 on it");
        }
        for (code, label) in cases {
            let lang = crate::i18n::Lang::from_code(code).unwrap_or(crate::i18n::Lang::EN);
            let mut h = egui_kittest::Harness::builder().build_ui_state(
                move |ui, count: &mut usize| {
                    crate::i18n::with_language(lang, || {
                        if theo_button(ui, tl!("Ask Theo for help"), "Theo").clicked() {
                            *count += 1;
                        }
                    });
                },
                0usize,
            );
            h.get_by_label(label).click();
            h.run_steps(2);
            assert_eq!(*h.state(), 1, "{code}");
        }
        let decoded = photocraft_codecs::decode(include_bytes!("../../../assets/zenda/theo-icon-64.png")).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (64, 64));
    }

    #[test]
    fn zenda_title_bar_theo_reaches_work() {
        use egui_kittest::kittest::Queryable;
        // The whole app, so the button is drawn in the real title bar with PhotoCraft's fonts.
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).with_max_steps(64).build_eframe(|cc| {
            PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            app()
        });
        h.run_steps(4);
        if !ON {
            assert!(h.query_by_label("Ask Theo for help").is_none());
            return;
        }
        h.get_by_label("Ask Theo for help").click();
        h.run_steps(2);
        let events = h.state_mut().zenda.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event"], "work.theo");
        assert_eq!(events[0]["params"]["from"], "button");
    }

    #[test]
    fn zenda_about_names_the_base_app_without_artcraft_links() {
        let source = include_str!("dialogs.rs");
        assert!(source.contains("Based on PhotoCraft, by the ArtCraft team."));
    }
}
