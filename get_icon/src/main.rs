use image::{ImageBuffer, Rgba};

use base64::{engine::general_purpose, engine::general_purpose::STANDARD as BASE64, Engine as _};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};

fn decode_base64(base64_data: &str, output_path: &str) -> std::io::Result<()> {
    let decode_data = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    // 创建 RGBA 图像
    let width = 32;
    let height = 32;
    let mut img = ImageBuffer::new(width, height);

    // 将解码后的数据转换为 RGBA 像素
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let idx = ((y * width + x) * 4) as usize;
        if idx + 3 < decode_data.len() {
            *pixel = Rgba([
                decode_data[idx + 2], // R
                decode_data[idx + 1], // G
                decode_data[idx],     // B
                decode_data[idx + 3], // A
            ]);
        }
    }

    // 保存为 PNG
    img.save(output_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

fn get_icon_as_base64(exe_path: &str) -> Result<String> {
    unsafe {
        // 加载可执行文件
        let h_module = LoadLibraryExW(
            PCWSTR::from_raw(
                exe_path
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            HANDLE(0),
            LOAD_LIBRARY_AS_DATAFILE,
        )?;

        // 获取图标
        let icon = ExtractIconW(
            h_module,
            PCWSTR::from_raw(
                exe_path
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            0,
        );

        if icon.is_invalid() {
            return Err(Error::from_win32());
        }

        // 获取图标信息
        let mut icon_info = ICONINFO::default();
        if !GetIconInfo(icon, &mut icon_info).as_bool() {
            return Err(Error::from_win32());
        }

        // 获取位图信息
        let mut bitmap_info = BITMAP::default();
        GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap_info as *mut _ as *mut std::ffi::c_void),
        );

        // 创建位图
        let hdc = GetDC(HWND(0));
        let hdc_mem = CreateCompatibleDC(hdc);
        let h_bitmap = CreateCompatibleBitmap(hdc, bitmap_info.bmWidth, bitmap_info.bmHeight);
        let old_bitmap = SelectObject(hdc_mem, h_bitmap);

        // 绘制图标到位图
        DrawIcon(hdc_mem, 0, 0, icon);

        // 获取位图数据
        let mut buffer = vec![0u8; (bitmap_info.bmWidth * bitmap_info.bmHeight * 4) as usize];
        let mut bitmap_info_header = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: bitmap_info.bmWidth,
            biHeight: -bitmap_info.bmHeight, // 负值表示自上而下
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        GetDIBits(
            hdc_mem,
            h_bitmap,
            0,
            bitmap_info.bmHeight as u32,
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bitmap_info_header as *mut _ as *mut BITMAPINFO,
            DIB_RGB_COLORS,
        );

        // 清理资源
        SelectObject(hdc_mem, old_bitmap);
        DeleteObject(h_bitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(HWND(0), hdc);
        DestroyIcon(icon);

        // 转换为base64
        let base64_string = BASE64.encode(&buffer);
        Ok(base64_string)
    }
}

fn get_process_id() -> Result<u32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Err(Error::new(E_FAIL, HSTRING::from("无法获取前台窗口")));
        }

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        Ok(pid)
    }
}

fn main() -> Result<()> {
    let exe_path = r"C:\Program Files\Tencent\WeChat\WeChat.exe";
    match get_icon_as_base64(exe_path) {
        Ok(base64_string) => {
            println!("图标已成功转换为base64格式");

            // 验证base64
            let output_path = "icon.png";
            match decode_base64(&base64_string, output_path) {
                Ok(_) => println!("验证成功：图标已保存为 {}", output_path),
                Err(e) => println!("验证失败：{}", e),
            }
        }
        Err(e) => {
            println!("错误：{}", e);
        }
    }

    let pid = get_process_id()?;
    println!("当前前台应用的进程 ID 是：{}", pid);

    Ok(())
}
