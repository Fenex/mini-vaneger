use druid::{widget::*, Widget};

use crate::state::{AddonsCfg, Item};

pub fn ui() -> impl Widget<AddonsCfg> {
    List::new(item)
        .lens(AddonsCfg::addons)
        .scroll()
        .vertical()
        .controller(AddonCfgCtrl)
}

fn item() -> impl Widget<Item> {
    Flex::row()
        .with_child(Checkbox::new("").lens(Item::enabled))
        .with_flex_child(
            Label::dynamic(|data: &String, _| data.clone())
                .with_text_size(18.)
                .expand_width()
                .lens(Item::name),
            1.0,
        )
        .padding(10.0)
}

struct AddonCfgCtrl;

impl<W: Widget<AddonsCfg>> Controller<AddonsCfg, W> for AddonCfgCtrl {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &AddonsCfg,
        data: &AddonsCfg,
        env: &druid::Env,
    ) {
        for (a, b) in old_data.addons.iter().zip(data.addons.iter()) {
            if a != b {
                data.save();
                break;
            }
        }

        child.update(ctx, old_data, data, env)
    }
}
