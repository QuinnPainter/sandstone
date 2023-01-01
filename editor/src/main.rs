mod ui;

use imgui::{WindowFlags, Condition};

fn main() {
    ui::mainloop(move |ui| {
        //ui.show_demo_window(&mut true);
        // seems that imgui-rs has no abstraction for this yet, so we must use the raw binding
        unsafe {
            let view = imgui::sys::igGetMainViewport();
            let win_class = imgui::sys::ImGuiWindowClass_ImGuiWindowClass();
            imgui::sys::igDockSpaceOverViewport(view, 0, win_class);
        }
        ui.main_menu_bar(|| {
            let _ = ui.menu_item("Stuff");
            let _ = ui.menu_item("things");
        });

        /*let (view_pos, view_size) = unsafe {
            let view = imgui_sys::igGetMainViewport();
            ((*view).WorkPos, (*view).WorkSize)
        };
        let style_no_rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));
        let style_no_border = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
        ui.window("Main")
            .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_COLLAPSE | WindowFlags::NO_RESIZE |
                WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | WindowFlags::NO_NAV_FOCUS |
                WindowFlags::MENU_BAR | WindowFlags::NO_DOCKING)
            .size([view_size.x, view_size.y], Condition::Always)
            .position([view_pos.x, view_pos.y], Condition::Always)
            .build(|| {
                unsafe {
                }
            });
        style_no_border.pop();
        style_no_rounding.pop();*/
        ui.window("Inspector")
            .size([100.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Gday\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\neef");
            });
        ui.window("Hierarchy")
            .size([100.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("mate");
            });
    });
}
