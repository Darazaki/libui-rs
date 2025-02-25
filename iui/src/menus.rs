//! Menus that appear at the top of windows, and the items that go in them.

use controls::Window;
use libc::{c_int, c_void};
use std::ffi::CString;
use std::mem;
use ui_sys::{self, uiMenu, uiMenuItem, uiWindow};
use UI;

/// A `MenuItem` represents an item that is shown in a `Menu`. Note that, unlike many controls,
/// the text on `MenuItem`s cannot be changed after creation.
#[derive(Clone)]
pub struct MenuItem {
    ui_menu_item: *mut uiMenuItem,
}

/// A `Menu` represents one of the top-level menus at the top of a window. As that bar is unique
/// per application, creating a new `Menu` shows it on all windows that support displaying menus.
#[derive(Clone)]
pub struct Menu {
    ui_menu: *mut uiMenu,
}

impl MenuItem {
    /// Enables the item, allowing it to be selected. This is the default state of a menu item.
    pub fn enable(&self, _ctx: &UI) {
        unsafe { ui_sys::uiMenuItemEnable(self.ui_menu_item) }
    }

    /// Disables the item, preventing it from being selected and providing a visual cue to the
    /// user that it cannot be selected.
    pub fn disable(&self, _ctx: &UI) {
        unsafe { ui_sys::uiMenuItemDisable(self.ui_menu_item) }
    }

    /// Returns `true` if the menu item is checked, and false if it is not checked (or not checkable).
    pub fn checked(&self, _ctx: &UI) -> bool {
        unsafe { ui_sys::uiMenuItemChecked(self.ui_menu_item) != 0 }
    }

    /// Sets the menu item to either checked or unchecked based on the given value.
    ///
    /// Setting the checked value of a non-checkable menu item has no effect.
    pub fn set_checked(&self, _ctx: &UI, checked: bool) {
        unsafe { ui_sys::uiMenuItemSetChecked(self.ui_menu_item, checked as c_int) }
    }

    /// Sets the function to be executed when the item is clicked/selected.
    pub fn on_clicked<F: FnMut(&MenuItem, &Window)>(&self, _ctx: &UI, callback: F) {
        unsafe {
            let mut data: Box<Box<dyn FnMut(&MenuItem, &Window)>> = Box::new(Box::new(callback));
            ui_sys::uiMenuItemOnClicked(
                self.ui_menu_item,
                c_callback,
                &mut *data as *mut Box<dyn FnMut(&MenuItem, &Window)> as *mut c_void,
            );
            mem::forget(data);
        }

        extern "C" fn c_callback(
            menu_item: *mut uiMenuItem,
            window: *mut uiWindow,
            data: *mut c_void,
        ) {
            unsafe {
                let menu_item = MenuItem {
                    ui_menu_item: menu_item,
                };
                let window = Window::from_raw(window);
                mem::transmute::<*mut c_void, &mut Box<dyn FnMut(&MenuItem, &Window)>>(data)(
                    &menu_item, &window,
                );
                mem::forget(window);
            }
        }
    }

    // Creates a `MenuItem` from a raw pointer
    pub unsafe fn from_raw(raw: *mut uiMenuItem) -> Self {
        MenuItem { ui_menu_item: raw }
    }
}

impl Menu {
    /// Creates a new menu with the given name to be displayed in the menubar at the top of the window.
    pub fn new(_ctx: &UI, name: &str) -> Menu {
        unsafe {
            let c_string = CString::new(name.as_bytes().to_vec()).unwrap();
            Menu {
                ui_menu: ui_sys::uiNewMenu(c_string.as_ptr()),
            }
        }
    }

    /// Adds a new item with the given name to the menu.
    pub fn append_item(&self, name: &str) -> MenuItem {
        unsafe {
            let c_string = CString::new(name.as_bytes().to_vec()).unwrap();
            MenuItem {
                ui_menu_item: ui_sys::uiMenuAppendItem(self.ui_menu, c_string.as_ptr()),
            }
        }
    }

    /// Adds a new togglable (checkbox) item with the given name to the menu.
    pub fn append_check_item(&self, name: &str) -> MenuItem {
        unsafe {
            let c_string = CString::new(name.as_bytes().to_vec()).unwrap();
            MenuItem {
                ui_menu_item: ui_sys::uiMenuAppendCheckItem(self.ui_menu, c_string.as_ptr()),
            }
        }
    }

    /// Adds a seperator to the menu.
    pub fn append_separator(&self) {
        unsafe { ui_sys::uiMenuAppendSeparator(self.ui_menu) }
    }
}
