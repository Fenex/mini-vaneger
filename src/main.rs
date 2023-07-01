// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::widget::{Button, Flex, Label, Tabs};
use druid::{
    AppLauncher, Application, Data, Env, EventCtx, Selector, Widget, WidgetExt, WindowDesc,
};
use log::*;
use state::AppState;

mod settings;
mod state;
mod tab;
mod tabs_manager;

use tabs_manager::*;

pub const BTN_RUN_CLICKED: Selector<()> = Selector::new("button `run` was clicked");

pub fn main() {
    env_logger::init();

    // create the initial app state
    let initial_state = AppState::new();
    trace!("{:?}", initial_state);

    // describe the main window
    let main_window = WindowDesc::new(ui())
        // .resizable(false)
        // .window_size((500, 400))
        // .set_position((-1000., 50.))
        .title("vangers mini mod manager")
        .window_size((700.0, 400.0));

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn ui() -> impl Widget<AppState> {
    let tabs = Tabs::for_policy(VssTabs)
        .with_axis(TAB_AXIS)
        .with_edge(TAB_EDGE)
        .with_transition(TAB_TRANSITION)
        .controller(TabsCtrl);

    let buttons = Flex::row()
        .with_child(
            Button::from_label(Label::new("Run").with_text_size(25.))
                .on_click(onclick_button_run)
                .disabled_if(|d: &AppState, _| !d.settings.resource_dir_validated),
        )
        .with_spacer(20.)
        .with_child(
            Button::from_label(Label::new("Exit").with_text_size(25.))
                .on_click(|_, _, _| Application::global().quit()),
        )
        .align_left();

    Flex::column()
        .with_flex_child(tabs, 1.0)
        .with_child(block("Settings:", settings::ui().lens(AppState::settings)))
        .with_spacer(5.)
        .with_child(buttons)
        .padding(10.)
    // .controller(MainController)
}

fn block<W: Widget<T> + 'static, T: Data>(name: &str, inner: W) -> impl Widget<T> {
    Flex::column()
        .with_child(Label::new(name).with_text_size(22.0).align_left())
        .with_child(inner.padding((15., 0., 0., 0.)))
}

/// Обработчик кнопки запуска игры
fn onclick_button_run(ctx: &mut EventCtx, data: &mut AppState, _env: &Env) {
    if !data.settings.resource_dir.is_empty() && data.settings.resource_dir_validated {
        match dunce::canonicalize(&data.settings.resource_dir) {
            Ok(ref resource_dir) if resource_dir.is_dir() => {
                ctx.submit_command(BTN_RUN_CLICKED);
            }
            _ => (),
        }
    }

    ctx.set_handled();
}
