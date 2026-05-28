use base64::Engine;
use image::ImageEncoder;
use qrcode::QrCode;

use super::error::CertificateError;

pub fn generate_qr_png(url: &str) -> Result<Vec<u8>, CertificateError> {
    let code = QrCode::new(url.as_bytes())
        .map_err(|e| CertificateError::QrGeneration(e.to_string()))?;

    let image_buffer = code
        .render::<image::Luma<u8>>()
        .min_dimensions(200, 200)
        .build();

    let mut png_bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            image_buffer.as_raw(),
            image_buffer.width(),
            image_buffer.height(),
            image::ColorType::L8.into(),
        )
        .map_err(|e| CertificateError::QrGeneration(e.to_string()))?;

    Ok(png_bytes)
}

pub fn generate_qr_data_url(url: &str) -> Result<String, CertificateError> {
    let png = generate_qr_png(url)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    Ok(format!("data:image/png;base64,{b64}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_png_is_non_empty() {
        let png = generate_qr_png("https://example.com/verify/CERT-2026-ABCD1234").unwrap();
        assert!(!png.is_empty());
        // PNG magic bytes: 0x89 0x50 0x4E 0x47
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47], "not a PNG");
    }

    #[test]
    fn qr_data_url_starts_with_prefix() {
        let data_url =
            generate_qr_data_url("https://example.com/verify/CERT-2026-ABCD1234").unwrap();
        assert!(data_url.starts_with("data:image/png;base64,"));
    }
}
