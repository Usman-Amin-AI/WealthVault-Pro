use log::error;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    let tray_menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("toggle_window", "Show / Hide").build(app).unwrap())
        .separator()
        .item(&MenuItemBuilder::with_id("open_settings", "Settings...").build(app).unwrap())
        .separator()
        .item(&PredefinedMenuItem::quit(app, None).unwrap())
        .build()
        .unwrap();

    let tray_result = TrayIconBuilder::new()
        .icon(Image::new(include_bytes!("../icons/32x32.png"), 32, 32))
        .menu(&tray_menu)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = match window.is_visible() {
                        Ok(true) => window.hide(),
                        _ => {
                            let _ = window.show();
                            let _ = window.set_focus();
                            Ok(())
                        }
                    };
                }
            }
        })
        .build(app);

    if let Err(e) = tray_result {
        error!("Failed to build tray icon: {}", e);
    }
}
