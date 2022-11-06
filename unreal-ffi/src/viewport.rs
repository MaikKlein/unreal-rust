#[repr(u32)]
pub enum MouseState {
    Visible,
    Hidden,
}

pub type LocalPlayerId = u32;

extern "C" {
    pub fn GetViewportSize(player: LocalPlayerId, x: *mut f32, y: *mut f32);
    pub fn SetMouseState(player: LocalPlayerId, state: MouseState);
    pub fn GetMousePosition(player: LocalPlayerId, x: *mut f32, y: *mut f32);
}
pub type GetViewportSizeFn = unsafe extern "C" fn(player: LocalPlayerId, x: *mut f32, y: *mut f32);
pub type SetMouseStateFn = unsafe extern "C" fn(player: LocalPlayerId, state: MouseState);
pub type GetMousePositionFn = unsafe extern "C" fn(player: LocalPlayerId, x: *mut f32, y: *mut f32);

#[repr(C)]
pub struct ViewportFns {
    pub get_viewport_size: GetViewportSizeFn,
    pub set_mouse_state: SetMouseStateFn,
    pub get_mouse_position: GetMousePositionFn,
}
