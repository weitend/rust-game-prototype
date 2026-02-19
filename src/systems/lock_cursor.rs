use bevy::{
    ecs::system::Single,
    window::{CursorGrabMode, CursorOptions},
};

pub fn lock_cursor_system(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}
