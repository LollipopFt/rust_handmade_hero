
use windows::{
    Win32::{
        UI::WindowsAndMessaging::*,
        Graphics::Gdi::*,
        Foundation::{PSTR, HWND, WPARAM, LPARAM, LRESULT, RECT},
        System::{
            LibraryLoader::GetModuleHandleA,
            Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE}
        }
    }
};

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
static mut BITMAP_MEMORY: *mut std::ffi::c_void = std::ptr::null_mut::<std::ffi::c_void>();
static mut BITMAP_WIDTH: i32 = 0;
static mut BITMAP_HEIGHT: i32 = 0;
const BYTES_PER_PIXEL: i32 = 4;

fn render_weird_gradient(x_offset: i32, y_offset: i32) {
unsafe {
    let width = BITMAP_WIDTH;
    let pitch = width*BYTES_PER_PIXEL;
    let mut row = BITMAP_MEMORY as *mut u8;
    for y in 0..BITMAP_HEIGHT {
        let mut pixel = row as *mut u32;
        for x in 0..BITMAP_WIDTH {
            let blue = (x+x_offset) as u32;
            let green = (y+y_offset) as u32;
            *pixel = ((green & 0xFF) << 8) | (blue & 0xFF);
            pixel = pixel.offset(1);
        }
        row = row.offset(pitch as isize);
    }
}
}

fn resize_dib_section(width: i32, height: i32) {
unsafe {
    if !BITMAP_MEMORY.is_null() {
        VirtualFree(BITMAP_MEMORY, 0, MEM_RELEASE);
    }

    BITMAP_WIDTH = width;
    BITMAP_HEIGHT = height;

    BITMAP_INFO.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    BITMAP_INFO.bmiHeader.biWidth = BITMAP_WIDTH;
    BITMAP_INFO.bmiHeader.biHeight = -BITMAP_HEIGHT;
    BITMAP_INFO.bmiHeader.biPlanes = 1;
    BITMAP_INFO.bmiHeader.biBitCount = 32;
    BITMAP_INFO.bmiHeader.biCompression = BI_RGB as u32;

    let bitmap_memory_size = BITMAP_WIDTH*BITMAP_HEIGHT*BYTES_PER_PIXEL;
    BITMAP_MEMORY = VirtualAlloc(
        std::ptr::null::<std::ffi::c_void>(),
        bitmap_memory_size as usize,
        MEM_COMMIT,
        PAGE_READWRITE
    );
}
}

fn update_window(device_context: HDC, client_rect: RECT) {
    let window_width = client_rect.right - client_rect.left;
    let window_height = client_rect.bottom - client_rect.top;
unsafe {
    StretchDIBits(
        device_context,
        0, 0, BITMAP_WIDTH, BITMAP_HEIGHT,
        0, 0, window_width, window_height,
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
            let mut client_rect: RECT = Default::default();
            GetClientRect(window, &mut client_rect);
            update_window(device_context, client_rect);
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
unsafe {
    RegisterClassA(&window_class);
    let window = CreateWindowExA(
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
    if window.0 > 0 {
        let mut x_offset = 0;
        let y_offset = 0;
        while RUNNING {
            let mut message = MSG::default();
            while PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into() {
                if message.message == WM_QUIT {
                    RUNNING = false;
                }
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
            render_weird_gradient(x_offset, y_offset);
            let device_context: HDC = GetDC(window);
            let mut client_rect: RECT = Default::default();
            GetClientRect(window, &mut client_rect);
            update_window(device_context, client_rect);
            ReleaseDC(window, device_context);

            x_offset+=1;
        }
    }
}
}
