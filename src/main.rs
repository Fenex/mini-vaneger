mod state;

use std::{fs::{File, canonicalize}, path::{Path, PathBuf}, process};

use druid::{
    widget::{Button, Checkbox, Controller, Flex, Label, List, Scroll, TextBox},
    AppLauncher, Application, LensExt, PlatformError, Selector, Widget, WidgetExt, WindowDesc, EventCtx, Env, FileDialogOptions, FileInfo, commands,
};

use state::*;

pub const BTN_RUN_CLICKED: Selector<()> = Selector::new("button `run` was clicked");
pub const BTN_RESOURCE_CHOOSE_CLICKED: Selector<()> = Selector::new("button `choose resources` was clicked");
pub const RESOURCES_DIR_CHOOSEN: Selector<FileInfo> = Selector::new("RESOURCES_DIR_CHOOSEN");

const CONFIG_FILE_PATH: &str = "./scripts/ls.json";
const PATH_TO_EXECUTE_PROGRAM: &str = "notepad.exe";

pub fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        // .resizable(false)
        // .window_size((500, 400))
        .set_position((-1000.,50.))
        .window_size((400., 300.))
        .title("vangers mini mod manager");

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(AppState::new(Path::new(CONFIG_FILE_PATH)))
}

fn ui_builder() -> impl Widget<AppState> {
    let scroll = Scroll::new(List::new(item)).vertical()
        .lens(AppState::config.then(Config::addons));
    let buttons = Flex::row()
        .with_child(
            Button::from_label(Label::new("Run").with_text_size(25.)).on_click(|ctx, _, _| {
                ctx.submit_notification(BTN_RUN_CLICKED);
            })
            .disabled_if(|d: &AppState, _| d.resource_dir.is_none()),
        )
        .with_spacer(20.)
        .with_child(
            Button::from_label(Label::new("Exit").with_text_size(25.))
                .on_click(|_, _, _| Application::global().quit()),
        )
        .align_left();

    Flex::column()
        .with_flex_child(scroll, 1.0)
        .with_child(resource_dir_selector().lens(AppState::resource_dir))
        .with_child(buttons)
        .padding(10.)
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
                if let Some(ref dir)= data.resource_dir {
                    let path = canonicalize(dir).unwrap();
                    let ls_json_path = canonicalize(&data.config.path).unwrap();
                    let ls_json = ls_json_path.parent().unwrap().to_string_lossy().to_string();
                    let vss = ls_json_path.parent().and_then(|p| p.parent()).map(|p| p.to_owned().join("vss.exe")).unwrap();
                    let _ = process::Command::new(&vss)
                        .current_dir(path)
                        .args(["-vss", &dunce::canonicalize(ls_json).unwrap().to_string_lossy(), "-russian"])
                        .spawn()
                        .expect("Cannot exec process");
                    ctx.set_handled();

                }
            },
            druid::Event::Command(e) if e.is(RESOURCES_DIR_CHOOSEN) => {
                let finfo = e.get_unchecked(RESOURCES_DIR_CHOOSEN);
                let check = finfo.path.join("tabutask.prm");
                if check.exists() {
                    data.resource_dir = Some(finfo.path.to_string_lossy().to_string())
                } else {
                    // TODO: notif incorrect path to resource folder
                }
                ctx.set_handled();
            },
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

fn resource_dir_selector() -> impl Widget<Option<String>> {
    Flex::row()
        .with_child(Label::new("Path to resources:"))
        .with_flex_child(Label::dynamic(|d: &Option<String>, _| d.clone().unwrap_or_default()), 1.0)
        .with_child(Button::new("...").on_click(dlg_choose_resources))
}

fn dlg_choose_resources(ctx: &mut EventCtx, data: &mut Option<String>, env: &Env) {
    let fdialog = FileDialogOptions::new()
        .accept_command(RESOURCES_DIR_CHOOSEN)
        .select_directories()
        // .cancel_command(cmd)
        .button_text("...");
    ctx.submit_command(commands::SHOW_OPEN_PANEL.with(fdialog));
}

#[cfg(test)]
mod test {
    #[test]
    fn qq() {
        let c = std::env::current_exe().unwrap().canonicalize().unwrap();
        dbg!(&c);
        let aa = dunce::canonicalize(c).unwrap();

        dbg!(&aa);

    }
}