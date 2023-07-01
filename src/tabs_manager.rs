use std::{path::Path, process, time::Duration};

use druid::{widget::*, Application, Data, LensExt, Widget};
use log::{error, trace};

use crate::{
    state::{AppState, Language},
    tab, BTN_RUN_CLICKED,
};

pub const TAB_AXIS: Axis = Axis::Vertical;
pub const TAB_EDGE: TabsEdge = TabsEdge::Leading;
pub const TAB_TRANSITION: TabsTransition =
    TabsTransition::Slide(Duration::from_millis(180).as_nanos() as u64);

#[derive(Clone, Data)]
pub struct VssTabs;

impl TabsPolicy for VssTabs {
    type Key = usize;
    type Input = AppState;
    type BodyWidget = Box<dyn Widget<Self::Input>>;
    type LabelWidget = Label<Self::Input>;
    type Build = ();

    fn tabs_changed(&self, _old_data: &Self::Input, _data: &Self::Input) -> bool {
        false
    }

    fn tabs(&self, data: &Self::Input) -> Vec<Self::Key> {
        data.vss
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            // std::iter::empty()
            //     .chain([0])
            //     .chain(data.vss.iter().enumerate().map(|(i, _)| (i+1) as usize))
            .collect()
    }

    fn tab_info(&self, key: Self::Key, data: &Self::Input) -> TabInfo<Self::Input> {
        TabInfo::new(&**data.vss[key].dirname, false)
        // match key {
        //     0 => TabInfo::new("Settings", false),
        //     k => TabInfo::new(&**data.vss[k - 1].dirname, false)
        // }
    }

    fn tab_body(&self, key: Self::Key, _data: &Self::Input) -> Self::BodyWidget {
        tab::ui().lens(AppState::vss.index(key)).boxed()
        // match key {
        //     0 => crate::settings::ui().lens(AppState::settings).boxed(),
        //     k => addon_tab().lens(AppState::vss.index(k - 1)).boxed()
        // }
    }

    fn tab_label(
        &self,
        _key: Self::Key,
        info: TabInfo<Self::Input>,
        _data: &Self::Input,
    ) -> Self::LabelWidget {
        Self::default_make_label(info)
    }
}

pub struct TabsCtrl;

impl Controller<AppState, Tabs<VssTabs>> for TabsCtrl {
    fn event(
        &mut self,
        child: &mut Tabs<VssTabs>,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &druid::Env,
    ) {
        if let druid::Event::Command(cmd) = event {
            if cmd.is(BTN_RUN_CLICKED) {
                let cfg = &data.vss[child.tab_index()];
                let working_directory = Path::new(&data.settings.resource_dir);
                trace!("working directory is `{:?}`", &working_directory);
                let addon_directory = Path::new(&*cfg.config_path).parent().unwrap();
                trace!("addon directory is `{:?}`", &addon_directory);

                let proccess_vss = process::Command::new(addon_directory.join("vss.exe"))
                    .current_dir(working_directory)
                    .arg("-vss")
                    .arg(addon_directory.join("scripts"))
                    .arg(match data.settings.language {
                        Language::En => "",
                        Language::Ru => "-russian",
                    })
                    .spawn();

                ctx.set_handled();

                if let Err(e) = proccess_vss {
                    error!("cannot run vss.exe: {:?}", e);
                } else {
                    Application::global().quit();
                }
            }
        }

        child.event(ctx, event, data, env)
    }
}
