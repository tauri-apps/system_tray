use std::sync::Arc;

use arc_swap::ArcSwap;
use muda::{AboutDialog, PredefinedMenuItemKind};

use super::tray::Tray;

pub fn muda_to_ksni_menu_item(
    item: Arc<ArcSwap<muda::CompatMenuItem>>,
) -> ksni::menu::MenuItem<Tray> {
    match &**item.load() {
        muda::CompatMenuItem::Standard(menu_item) => {
            let id = menu_item.id.clone();
            match &menu_item.predefined_menu_item_kind {
                Some(PredefinedMenuItemKind::About(Some(metadata))) => {
                    let about_dialog = AboutDialog::new(metadata.clone());
                    ksni::menu::StandardItem {
                        label: menu_item.label.clone(),
                        enabled: menu_item.enabled,
                        icon_data: menu_item.icon.clone().unwrap_or_default(),
                        activate: Box::new(move |_| {
                            about_dialog.show();
                        }),
                        ..Default::default()
                    }
                    .into()
                }
                _ => ksni::menu::StandardItem {
                    label: menu_item.label.clone(),
                    enabled: menu_item.enabled,
                    icon_data: menu_item.icon.clone().unwrap_or_default(),
                    activate: Box::new(move |_| send_menu_event(&id)),
                    ..Default::default()
                }
                .into(),
            }
        }
        muda::CompatMenuItem::Checkmark(check_menu_item) => {
            let id = check_menu_item.id.clone();
            ksni::menu::CheckmarkItem {
                label: check_menu_item.label.clone(),
                enabled: check_menu_item.enabled,
                checked: check_menu_item.checked,
                activate: Box::new(move |_| send_menu_event(&id)),
                ..Default::default()
            }
            .into()
        }
        muda::CompatMenuItem::SubMenu(submenu) => ksni::menu::SubMenu {
            label: submenu.label.clone(),
            enabled: submenu.enabled,
            submenu: submenu
                .submenu
                .iter()
                .cloned()
                .map(muda_to_ksni_menu_item)
                .collect(),
            ..Default::default()
        }
        .into(),
        muda::CompatMenuItem::Separator => ksni::menu::MenuItem::Separator,
    }
}

fn send_menu_event(id: &str) {
    muda::MenuEvent::send(muda::MenuEvent {
        id: muda::MenuId(id.to_string()),
    })
}
