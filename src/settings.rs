use druid::{widget::*, *};

use crate::state::{
    Language, ResourceDirectoryState, Settings2ResourceDirectoryState, SettingsCfg,
};

const EXCL: &[u8] = include_bytes!("../resources/excl_50.png");

const RESOURCES_DIR_CHOOSEN: Selector<FileInfo> = Selector::new("RESOURCES_DIR_CHOOSEN");

pub fn ui() -> impl Widget<SettingsCfg> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(
                    Label::new("Path to resources:").with_text_color(Color::rgb8(180, 180, 180)),
                )
                .with_flex_child(
                    Either::new(
                        |d: &ResourceDirectoryState, _| {
                            d.resource_dir.is_empty() || d.resource_dir_validated
                        },
                        Label::new(|d: &String, _env: &_| d.clone())
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
                            .with_child(Image::new(ImageBuf::from_data(EXCL).unwrap())),
                    ),
                    1.,
                )
                .with_child(
                    Button::new("...")
                        .on_click(dlg_choose_resources)
                        .padding((5., 0., 0., 0.)),
                )
                .lens(Settings2ResourceDirectoryState),
        )
        .with_child(
            Flex::row()
                .with_child(Label::new("Language:").with_text_color(Color::rgb8(180, 180, 180)))
                .with_flex_child(
                    RadioGroup::row(vec![("EN", Language::En), ("RU", Language::Ru)]),
                    1.0,
                )
                .lens(SettingsCfg::language),
        )
        .controller(SettingsCtrl)
}

fn dlg_choose_resources(ctx: &mut EventCtx, _data: &mut ResourceDirectoryState, _: &Env) {
    let fdialog = FileDialogOptions::new()
        .accept_command(RESOURCES_DIR_CHOOSEN)
        .select_directories()
        // .cancel_command(cmd)
        .button_text("...");
    ctx.submit_command(commands::SHOW_OPEN_PANEL.with(fdialog));
}

struct SettingsCtrl;

impl<W: Widget<SettingsCfg>> Controller<SettingsCfg, W> for SettingsCtrl {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut SettingsCfg,
        env: &druid::Env,
    ) {
        match event {
            druid::Event::Command(e) if e.is(RESOURCES_DIR_CHOOSEN) => {
                let finfo = e.get_unchecked(RESOURCES_DIR_CHOOSEN);
                data.resource_dir = finfo.path.to_string_lossy().to_string();

                let check = finfo.path.join("tabutask.prm");
                if check.exists() {
                    data.resource_dir_validated = true;
                    data.save();
                } else {
                    data.resource_dir_validated = false;
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
        old_data: &SettingsCfg,
        data: &SettingsCfg,
        env: &druid::Env,
    ) {
        if old_data.language != data.language {
            data.save();
        }

        child.update(ctx, old_data, data, env)
    }
}
