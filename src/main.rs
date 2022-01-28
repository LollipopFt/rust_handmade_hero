use windows::{
    Win32::{
        UI::{
            Input::{
                XboxController::*,
                KeyboardAndMouse::*
            },
            WindowsAndMessaging::*
        },
        Graphics::Gdi::*,
        Foundation::{PSTR, HWND, WPARAM, LPARAM, LRESULT, RECT, ERROR_SUCCESS, HINSTANCE},
        System::{
            LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA},
            Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE}
        }
    }
};

#[derive(Clone, Copy)]
struct OffscreenBuffer {
    info: BITMAPINFO,
    memory: *mut std::ffi::c_void,
    width: i32,
    height: i32,
    pitch: i32,
    bytes_per_pixel: i32
}

static mut GLOBAL_RUNNING: bool = true;
static mut GLOBAL_BACKBUFFER: OffscreenBuffer = OffscreenBuffer {
    info: BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: 0,
            biWidth: 0,
            biHeight: 0,
            biPlanes: 0,
            biBitCount: 0,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0
        },
    bmiColors: [RGBQUAD {rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0}]
    },
    memory: std::ptr::null_mut::<std::ffi::c_void>(),
    width: 0,
    height: 0,
    pitch: 0,
    bytes_per_pixel: 0
};

struct WindowDimension {
    width: i32,
    height: i32
}

type Xinputgetstate = extern "system" fn(u32, *mut XINPUT_STATE) -> u32;
extern "system" fn xinput_get_state_stub(_: u32, _: *mut XINPUT_STATE) -> u32 {
    0
}
#[allow(non_upper_case_globals)]
static mut XInputGetState: Xinputgetstate = xinput_get_state_stub;

type Xinputsetstate = extern "system" fn(u32, *mut XINPUT_VIBRATION) -> u32;
extern "system" fn xinput_set_state_stub(_: u32, _: *mut XINPUT_VIBRATION) -> u32 {
    0
}
#[allow(non_upper_case_globals)]
static mut XInputSetState: Xinputsetstate = xinput_set_state_stub;

unsafe fn load_xinput() {
    let xinput_library: HINSTANCE = LoadLibraryA(PSTR(b"xinput1_3.dll\0".as_ptr() as _));
    if xinput_library != HINSTANCE(0) {
        XInputGetState = std::mem::transmute(
            GetProcAddress(xinput_library, PSTR(b"XInputGetState\0".as_ptr() as _))
        );
        XInputSetState = std::mem::transmute(
            GetProcAddress(xinput_library, PSTR(b"XInputSetState\0".as_ptr() as _))
        );
    }
}

fn get_window_dimension(window: HWND) -> WindowDimension {
    let mut result = WindowDimension { width: 0, height: 0 };
    let mut client_rect: RECT = Default::default();
    unsafe { GetClientRect(window, &mut client_rect) };
    result.width = client_rect.right - client_rect.left;
    result.height = client_rect.bottom - client_rect.top;
    result
}

fn render_weird_gradient(buffer: &mut OffscreenBuffer, blue_offset: i32, green_offset: i32) {
unsafe {
    let mut row = buffer.memory as *mut u8;
    for green in 0..buffer.height {
        let mut pixel = row as *mut u32;
        for blue in 0..buffer.width {
            let blue = (blue+blue_offset) as u32;
            let green = (green+green_offset) as u32;
            *pixel = ((green & 0xFF) << 8) | (blue & 0xFF);
            pixel = pixel.offset(1);
        }
        row = row.offset(buffer.pitch as isize);
    }
}
}

fn resize_dib_section(buffer: &mut OffscreenBuffer, width: i32, height: i32) {
unsafe {
    if !buffer.memory.is_null() {
        VirtualFree(buffer.memory, 0, MEM_RELEASE);
    }

    buffer.width = width;
    buffer.height = height;
    buffer.bytes_per_pixel = 4;

    buffer.info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    buffer.info.bmiHeader.biWidth = buffer.width;
    buffer.info.bmiHeader.biHeight = -buffer.height;
    buffer.info.bmiHeader.biPlanes = 1;
    buffer.info.bmiHeader.biBitCount = 32;
    buffer.info.bmiHeader.biCompression = BI_RGB as u32;

    let bitmap_memory_size = buffer.width*buffer.height*buffer.bytes_per_pixel;
    buffer.memory = VirtualAlloc(
        std::ptr::null::<std::ffi::c_void>(),
        bitmap_memory_size as usize,
        MEM_COMMIT,
        PAGE_READWRITE
    );
    buffer.pitch = width*buffer.bytes_per_pixel;
}
}

fn display_buffer_in_window(buffer: &OffscreenBuffer, device_context: HDC, window_width: i32, window_height: i32) {
unsafe {
    // TODO: aspect ratio correction
    StretchDIBits(
        device_context,
        0, 0, window_width, window_height,
        0, 0, buffer.width, buffer.height,
        buffer.memory,
        &buffer.info,
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
        },
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
        },
        WM_CLOSE | WM_DESTROY => {
            GLOBAL_RUNNING = false;
        },
        WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYUP | WM_KEYDOWN => {
            let vk_code = wparam;
            let was_down: bool = lparam.0 & (1 << 30) != 0;
            let is_down: bool = lparam.0 & (1 << 31) == 0;
            if was_down != is_down {
                match vk_code.0 as u8 as char {
                    'W' => println!("W"),
                    'A' => println!("A"),
                    'S' => println!("S"),
                    'D' => println!("D"),
                    'Q' => println!("Q"),
                    'E' => println!("E"),
                    _ => match vk_code.0 as u16 {
                        // WPARAM(VK_UP) => ,
                        // WPARAM(VK_LEFT) => ,
                        // WPARAM(VK_DOWN) => ,
                        // WPARAM(VK_RIGHT) => ,
                        VK_ESCAPE => {
                            print!("ESCAPE: ");
                            if is_down {
                                print!("is_down ");
                            }
                            if was_down {
                                print!("was_down");
                            }
                            println!();
                        },
                        VK_SPACE => println!("SPACE"),
                        _ => {}
                    }
                }
            }
        }
        WM_PAINT => {
            let mut paint: PAINTSTRUCT = Default::default();
            let device_context: HDC = BeginPaint(window, &mut paint);
            let dimension: WindowDimension = get_window_dimension(window);
            display_buffer_in_window(&GLOBAL_BACKBUFFER, device_context, dimension.width, dimension.height);
            EndPaint(window, &paint);
        },
        _ => {
            result = DefWindowProcA(window, message, wparam, lparam);
        }
    }
}
    result
}

fn main() {

    unsafe { load_xinput() };

    let window_class = WNDCLASSA {
        style: CS_HREDRAW|CS_VREDRAW,
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

    unsafe { resize_dib_section(&mut GLOBAL_BACKBUFFER, 1280, 720) };

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
    if window.0 != 0 {
        let device_context: HDC = GetDC(window);
        let mut x_offset = 0;
        let mut y_offset = 0;
        while GLOBAL_RUNNING {
            let mut message = MSG::default();
            while PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into() {
                if message.message == WM_QUIT {
                    GLOBAL_RUNNING = false;
                }
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            for controller_index in 0..XUSER_MAX_COUNT {
                let mut controller_state: XINPUT_STATE = Default::default();
                if XInputGetState(controller_index, &mut controller_state) == ERROR_SUCCESS {
                    // NOTE: controller is plugged in
                    let pad: &XINPUT_GAMEPAD = &controller_state.Gamepad;
                    // let up = pad.wButtons & XINPUT_GAMEPAD_DPAD_UP as u16;
                    // let down = pad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN as u16;
                    // let left = pad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT as u16;
                    // let right = pad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT as u16;
                    // let start = pad.wButtons & XINPUT_GAMEPAD_START as u16;
                    // let back = pad.wButtons & XINPUT_GAMEPAD_BACK as u16;
                    // let left_shoulder = pad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER as u16;
                    // let right_shoulder = pad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER as u16;
                    // let a_button = pad.wButtons & XINPUT_GAMEPAD_A as u16;
                    // let b_button = pad.wButtons & XINPUT_GAMEPAD_B as u16;
                    // let x_button = pad.wButtons & XINPUT_GAMEPAD_X as u16;
                    // let y_button = pad.wButtons & XINPUT_GAMEPAD_Y as u16;

                    let stick_x = pad.sThumbLX;
                    let stick_y = pad.sThumbLY;

                    x_offset += stick_x as i32 >> 12;
                    y_offset += stick_y as i32 >> 12;
                } else {
                    // NOTE: controller is not available
                }
            }
            render_weird_gradient(&mut GLOBAL_BACKBUFFER, x_offset, y_offset);
            let dimension: WindowDimension = get_window_dimension(window);
            display_buffer_in_window(&GLOBAL_BACKBUFFER, device_context, dimension.width, dimension.height);
        }
    }
}
}