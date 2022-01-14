use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK, MB_ICONINFORMATION};

fn main() {
    unsafe {
        MessageBoxA(None, "This is Handmade Hero.", "Handmade Hero", MB_OK|MB_ICONINFORMATION);
    }
}