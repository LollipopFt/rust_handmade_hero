use windows::{
    Win32::{
        UI::WindowsAndMessaging::*,
        Graphics::Gdi::{HBRUSH, PAINTSTRUCT, HDC, BeginPaint, PatBlt, EndPaint, WHITENESS, ROP_CODE, BLACKNESS},
        Foundation::{PSTR, HWND, WPARAM, LPARAM, LRESULT},
        System::LibraryLoader::GetModuleHandleA
    }
};

fn main() {
    let window_class = WNDCLASSA {
        style: CS_OWNDC|CS_HREDRAW|CS_VREDRAW,
        lpfnWndProc: Some(main_window_fallback),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe {GetModuleHandleA(None)},
        hIcon: HICON(0),
        hCursor: HCURSOR(0),
        hbrBackground: HBRUSH(0),
        lpszMenuName: PSTR(&mut 0),
        lpszClassName: PSTR(b"HandmadeHeroWindowClass\0".as_ptr() as _),
    };
    let mut message = MSG::default();
    unsafe {
        RegisterClassA(&window_class);
        CreateWindowExA(
            0,
            "HandmadeHeroWindowClass",
            "Handmade Hero",
            WS_OVERLAPPEDWINDOW|WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            window_class.hInstance,
            std::ptr::null_mut()
        );
        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }
}

extern "system" fn main_window_fallback(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut result = LRESULT(0);
    let mut paint = PAINTSTRUCT {..Default::default()};
    unsafe {
        match message {
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
            },
            WM_PAINT => {
                println!("WM_PAINT");
                let device_context: HDC = BeginPaint(window, &mut paint);
                let x = paint.rcPaint.left;
                let y = paint.rcPaint.top;
                let width = paint.rcPaint.right - paint.rcPaint.left;
                let height = paint.rcPaint.bottom - paint.rcPaint.top;
                static mut OPERATION: ROP_CODE = WHITENESS;
                PatBlt(device_context, x, y, width, height, OPERATION);
                if OPERATION == WHITENESS {
                    OPERATION = BLACKNESS;
                } else {
                    OPERATION = WHITENESS;
                }
                EndPaint(window, &paint);
            },
            _=> {
                result = DefWindowProcA(window, message, wparam, lparam);
            }
        }
    }
    result
}