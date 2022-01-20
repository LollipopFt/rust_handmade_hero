use windows::{
    Win32::{
        UI::WindowsAndMessaging::*,
        Graphics::Gdi::*,
        Foundation::{PSTR, HWND, WPARAM, LPARAM, LRESULT, RECT},
        System::LibraryLoader::GetModuleHandleA
    }
};

// TODO: global
static mut RUNNING: bool = true;
static mut BITMAP_INFO: BITMAPINFO = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
        biSize: 0,
        biWidth: 0,
        biHeight: 0,
        biPlanes: 1,
        biBitCount: 0,
        biCompression: 0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0
    },
    bmiColors: [RGBQUAD {rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0}]
};
static mut BITMAP_MEMORY: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;
static mut BITMAP_HANDLE: HBITMAP = HBITMAP(0);
static mut BITMAP_DEVICE_CONTEXT: HDC = HDC(0);

fn resize_dib_section(width: i32, height: i32) {
    unsafe {
        if !BITMAP_HANDLE.is_invalid() {
            DeleteObject(BITMAP_HANDLE);
        } 
        if BITMAP_DEVICE_CONTEXT.is_invalid() {
            BITMAP_DEVICE_CONTEXT = HDC(CreateCompatibleDC(HDC(0)).0);
        }
        BITMAP_INFO.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        BITMAP_INFO.bmiHeader.biWidth = width;
        BITMAP_INFO.bmiHeader.biHeight = height;
        BITMAP_INFO.bmiHeader.biPlanes = 1;
        BITMAP_INFO.bmiHeader.biBitCount = 32;
        BITMAP_INFO.bmiHeader.biCompression = BI_RGB as u32;
        BITMAP_HANDLE = CreateDIBSection(
            BITMAP_DEVICE_CONTEXT,
            &BITMAP_INFO,
            DIB_RGB_COLORS,
            &mut BITMAP_MEMORY,
            None,
            0
        );
    }
}

fn update_window(device_context: HDC, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        StretchDIBits(
            device_context,
            x, y, width, height,
            x, y, width, height,
            BITMAP_MEMORY,
            &BITMAP_INFO,
            DIB_RGB_COLORS,
            SRCCOPY
        );
    }
}

extern "system" fn main_window_fallback(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut result = LRESULT(0);
    unsafe {
        match message {
            WM_SIZE => {
                let mut client_rect: RECT = Default::default();
                GetClientRect(window, &mut client_rect);
                let width: i32 = client_rect.right - client_rect.left;
                let height: i32 = client_rect.bottom - client_rect.top;
                resize_dib_section(width, height);
            },
            WM_CLOSE => {
                RUNNING = false;
            },
            WM_ACTIVATEAPP => {
                println!("WM_ACTIVATEAPP");
            },
            WM_DESTROY => {
                RUNNING = false;
            },
            WM_PAINT => {
                let mut paint: PAINTSTRUCT = Default::default();
                let device_context: HDC = BeginPaint(window, &mut paint);
                let x = paint.rcPaint.left;
                let y = paint.rcPaint.top;
                let width = paint.rcPaint.right - paint.rcPaint.left;
                let height = paint.rcPaint.bottom - paint.rcPaint.top;
                update_window(device_context, x, y, width, height);
                EndPaint(window, &paint);
            },
            _=> {
                result = DefWindowProcA(window, message, wparam, lparam);
            }
        }
    }
    result
}

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
        let window_handle = CreateWindowExA(
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
        if window_handle.0 > 0 {
            while RUNNING {
                if GetMessageA(&mut message, HWND(0), 0, 0).into() {
                    TranslateMessage(&message);
                    DispatchMessageA(&message);
                }                
            }
        }
    }
}
