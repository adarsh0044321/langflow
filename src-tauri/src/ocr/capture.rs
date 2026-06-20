use std::mem;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, SRCCOPY,
};
use image::{ImageBuffer, Rgba};

pub fn capture_screen_area(x: i32, y: i32, width: i32, height: i32) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if width <= 0 || height <= 0 {
        return Err("Invalid capture dimensions".into());
    }

    unsafe {
        // 1. Get Desktop DC and create a memory DC
        let hdc_screen = GetDC(HWND(0));
        let hdc_mem = CreateCompatibleDC(hdc_screen);

        // 2. Create compatible bitmap
        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        let hold = SelectObject(hdc_mem, hbitmap);

        // 3. BitBlt the screen contents into our memory DC
        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, x, y, SRCCOPY)?;

        // 4. Set up bitmap info for retrieval
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Negative height means top-down bitmap
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                biSizeImage: (width * height * 4) as u32,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: Default::default(),
        };

        // 5. Allocate buffer for BGRA pixel bytes
        let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        // 6. Get raw pixel bits
        GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Clean up GDI handles immediately
        SelectObject(hdc_mem, hold);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND(0), hdc_screen);

        // 7. Convert BGRA to RGBA bytes (Win32 format is BGRA, image crate expects RGBA)
        let mut rgba_buffer = vec![0; buffer.len()];
        for chunk_idx in 0..(buffer.len() / 4) {
            let offset = chunk_idx * 4;
            rgba_buffer[offset] = buffer[offset + 2];     // R
            rgba_buffer[offset + 1] = buffer[offset + 1]; // G
            rgba_buffer[offset + 2] = buffer[offset];     // B
            rgba_buffer[offset + 3] = buffer[offset + 3]; // A
        }

        // 8. Create ImageBuffer and encode to PNG in-memory
        let img_buf: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width as u32, height as u32, rgba_buffer)
                .ok_or("Failed to construct image buffer")?;

        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img_buf.write_to(&mut cursor, image::ImageFormat::Png)?;

        Ok(png_bytes)
    }
}
