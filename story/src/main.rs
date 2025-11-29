use story::StoryView;
use wirecrab::gpui::*;
use wirecrab::gpui_component::{Root, init};
use wirecrab::gui::assets::Assets;
use wirecrab::gui::{fonts, theme};

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        init(cx);
        theme::init(cx);

        if let Err(e) = fonts::register_with(cx.text_system().as_ref()) {
            eprintln!("Failed to register fonts: {}", e);
        }

        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Wirecrab Storybook".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| StoryView::new(window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap();
    });
}
