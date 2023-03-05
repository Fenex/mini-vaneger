mod state;

use std::{fs::File, path::Path, process};

use druid::{
    widget::{Button, Checkbox, Controller, Flex, Label, List, Scroll},
    AppLauncher, Application, LensExt, PlatformError, Selector, Widget, WidgetExt, WindowDesc,
};

use state::*;

pub const BTN_RUN_CLICKED: Selector<()> = Selector::new("button `run` was clicked");
const CONFIG_FILE_PATH: &str = "./config.json";
const PATH_TO_EXECUTE_PROGRAM: &str = "notepad.exe";

pub fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        // .resizable(false)
        // .window_size((500, 400))
        .title("vangers mini mod manager");

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(AppState::new(Path::new(CONFIG_FILE_PATH)))
}

fn ui_builder() -> impl Widget<AppState> {
    let scroll = Scroll::new(List::new(item)).vertical();
    let buttons = Flex::row()
        .with_child(
            Button::from_label(Label::new("Run").with_text_size(25.)).on_click(|ctx, _, _| {
                ctx.submit_notification(BTN_RUN_CLICKED);
            }),
        )
        .with_spacer(20.)
        .with_child(
            Button::from_label(Label::new("Exit").with_text_size(25.))
                .on_click(|_, _, _| Application::global().quit()),
        )
        .align_left();

    Flex::column()
        .with_flex_child(scroll, 1.0)
        .with_child(buttons)
        .padding(10.)
        .lens(AppState::config.then(Config::addons))
        .controller(MainController)
}

fn item() -> impl Widget<Item> {
    Flex::row()
        .with_child(Checkbox::new("").lens(Item::enabled))
        .with_flex_child(
            Label::dynamic(|data: &String, _| data.to_string())
                .with_text_size(20.)
                .expand_width()
                .lens(Item::name),
            1.0,
        )
        .padding(10.0)
}

struct MainController;

impl<W: Widget<AppState>> Controller<AppState, W> for MainController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &druid::Env,
    ) {
        match event {
            druid::Event::Notification(e) if e.is(BTN_RUN_CLICKED) => {
                let _ = process::Command::new(PATH_TO_EXECUTE_PROGRAM)
                    // .args(["-russian"])
                    .spawn()
                    .expect("Cannot exec process");
                ctx.set_handled();
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &druid::Env,
    ) {
        for (a, b) in old_data.config.addons.iter().zip(data.config.addons.iter()) {
            if a != b {
                let result = File::options()
                    .write(true)
                    .truncate(true)
                    .open(&data.config.path)
                    .map(|w| serde_json::to_writer_pretty(w, &data.config));

                if let Ok(Ok(())) = result {
                    println!("ok write");
                } else {
                    println!("err");
                }

                break;
            }
        }
        child.update(ctx, old_data, data, env)
    }
}
