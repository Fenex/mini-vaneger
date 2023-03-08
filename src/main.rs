mod state;
use state::*;

use std::{process};
use druid::{
    commands,
    widget::{Button, Checkbox, Controller, Either, Flex, Image, Label, List, Scroll},
    AppLauncher, Application, Color, Data, Env, EventCtx, FileDialogOptions, FileInfo, ImageBuf,
    LensExt, PlatformError, Selector, UnitPoint, Widget, WidgetExt, WindowDesc,
};

pub const BTN_RUN_CLICKED: Selector<()> = Selector::new("button `run` was clicked");
pub const BTN_RESOURCE_CHOOSE_CLICKED: Selector<()> =
    Selector::new("button `choose resources` was clicked");
pub const RESOURCES_DIR_CHOOSEN: Selector<FileInfo> = Selector::new("RESOURCES_DIR_CHOOSEN");

const EXCL: &'static [u8] = include_bytes!("../resources/excl_50.png");

pub fn main() -> Result<(), PlatformError> {
    env_logger::init();

    let main_window = WindowDesc::new(ui_builder())
        // .resizable(false)
        // .window_size((500, 400))
        // .set_position((-1000., 50.))
        .window_size((500., 300.))
        .title("vangers mini mod manager");

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(AppState::new())
}

fn ui_builder() -> impl Widget<AppState> {
    let scroll = Scroll::new(List::new(item))
        .vertical()
        .lens(AppState::config.then(AddonsCfg::addons));
    let buttons = Flex::row()
        .with_child(
            Button::from_label(Label::new("Run").with_text_size(25.))
                .on_click(|ctx, _, _| {
                    ctx.submit_notification(BTN_RUN_CLICKED);
                })
                .disabled_if(|d: &AppState, _| !d.settings.resource_dir_validated),
        )
        .with_spacer(20.)
        .with_child(
            Button::from_label(Label::new("Exit").with_text_size(25.))
                .on_click(|_, _, _| Application::global().quit()),
        )
        .align_left();

    Flex::column()
        .with_flex_child(block("Modifications:", scroll), 1.0)
        .with_child(block(
            "Settings:",
            settings().lens(AppState::settings.then(Settings2ResourceDirectoryState)),
        ))
        .with_spacer(5.)
        .with_child(buttons)
        .padding(10.)
        .controller(MainController)
    // .debug_paint_layout()
}

fn block<W: Widget<T> + 'static, T: Data>(name: &str, inner: W) -> impl Widget<T> {
    Flex::column()
        .with_child(Label::new(name).with_text_size(22.0).align_left())
        .with_child(inner.padding((15., 0., 0., 0.)))
        .align_vertical(UnitPoint::TOP_LEFT)
}

fn item() -> impl Widget<Item> {
    Flex::row()
        .with_child(Checkbox::new("").lens(Item::enabled))
        .with_flex_child(
            Label::dynamic(|data: &String, _| data.to_string())
                .with_text_size(18.)
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
                if !data.settings.resource_dir.is_empty() && data.settings.resource_dir_validated {
                    match dunce::canonicalize(&data.settings.resource_dir) {
                        Ok(ref resource_dir) if resource_dir.is_dir() => {
                            let vss = data
                                .config
                                .scripts_directory()
                                .parent()
                                .unwrap()
                                .join("vss.exe");
                            let _ = process::Command::new(&vss)
                                .current_dir(resource_dir)
                                .args([
                                    "-vss",
                                    data.config.scripts_directory().to_str().unwrap(),
                                    "-russian",
                                ])
                                .spawn()
                                .expect("Cannot exec process");
                            ctx.set_handled();
                            Application::global().quit();
                        }
                        _ => ctx.set_handled(),
                    }
                }
            }
            druid::Event::Command(e) if e.is(RESOURCES_DIR_CHOOSEN) => {
                let finfo = e.get_unchecked(RESOURCES_DIR_CHOOSEN);
                data.settings.resource_dir = finfo.path.to_string_lossy().to_string();

                let check = finfo.path.join("tabutask.prm");
                if check.exists() {
                    data.settings.resource_dir_validated = true;
                    data.settings.save();
                } else {
                    data.settings.resource_dir_validated = false;
                }
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
                data.config.save();
                break;
            }
        }
        child.update(ctx, old_data, data, env)
    }
}

fn settings() -> impl Widget<ResourceDirectoryState> {
    Flex::row()
        .with_child(Label::new("Path to resources:"))
        .with_flex_child(
            Either::new(
                |d: &ResourceDirectoryState, _| d.resource_dir.is_empty() || d.resource_dir_validated,
                Label::new(|d: &String, _env: &_| format!("{}", d))
                    .expand_width()
                    .lens(ResourceDirectoryState::resource_dir),
                Flex::row()
                .with_flex_child(
                    Label::new(|d: &String, _env: &_| d.clone())
                    .with_text_color(Color::RED)
                    .expand_width()
                    .lens(ResourceDirectoryState::resource_dir),
                    1.0,
                )
                .with_child(
                    Image::new(ImageBuf::from_data(EXCL).unwrap())
                )
            ),
            1.,
        )
        .with_child(Button::new("...").on_click(dlg_choose_resources).padding((5., 0., 0., 0.)))
}

fn dlg_choose_resources(ctx: &mut EventCtx, _data: &mut ResourceDirectoryState, _: &Env) {
    let fdialog = FileDialogOptions::new()
        .accept_command(RESOURCES_DIR_CHOOSEN)
        .select_directories()
        // .cancel_command(cmd)
        .button_text("...");
    ctx.submit_command(commands::SHOW_OPEN_PANEL.with(fdialog));
}
