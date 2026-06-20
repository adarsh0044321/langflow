use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{InMemoryRandomAccessStream, DataWriter};

pub async fn run_native_ocr(image_bytes: &[u8], _lang_code: Option<&str>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Create an in-memory stream for image bytes
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(image_bytes)?;
    writer.StoreAsync()?.await?;
    writer.FlushAsync()?.await?;
    stream.Seek(0)?;

    // 2. Decode the image into a SoftwareBitmap
    let decoder = BitmapDecoder::CreateAsync(&stream)?.await?;
    let bitmap = decoder.GetSoftwareBitmapAsync()?.await?;

    // 3. Initialize OcrEngine with user's preferred language or system default
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;

    // 4. Perform OCR
    let ocr_result = engine.RecognizeAsync(&bitmap)?.await?;
    
    // 5. Aggregate result text with line structures
    let mut text_lines = Vec::new();
    let lines = ocr_result.Lines()?;
    for line in lines {
        let line_text = line.Text()?.to_string();
        text_lines.push(line_text);
    }

    Ok(text_lines.join("\n"))
}
