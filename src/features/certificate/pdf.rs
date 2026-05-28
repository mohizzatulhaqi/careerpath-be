use chrono::{DateTime, Utc};
use printpdf::*;

use super::error::CertificateError;

pub struct CertificatePdfData {
    pub certificate_code: String,
    pub recipient_name: String,
    pub role_name: String,
    pub issued_at: DateTime<Utc>,
    pub modules_count: i32,
    pub verification_url: String,
    pub qr_code_png: Vec<u8>,
}

pub fn generate_pdf(data: &CertificatePdfData) -> Result<Vec<u8>, CertificateError> {
    // A4 landscape: 297 × 210 mm
    let (doc, page1, layer1) = PdfDocument::new(
        format!("Certificate {}", data.certificate_code),
        Mm(297.0),
        Mm(210.0),
        "Layer 1",
    );
    let layer = doc.get_page(page1).get_layer(layer1);

    // ── Fonts ─────────────────────────────────────────────────────────────────
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| CertificateError::PdfGeneration(e.to_string()))?;
    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| CertificateError::PdfGeneration(e.to_string()))?;
    let font_italic = doc
        .add_builtin_font(BuiltinFont::HelveticaOblique)
        .map_err(|e| CertificateError::PdfGeneration(e.to_string()))?;

    // ── Color palette ─────────────────────────────────────────────────────────
    let navy  = Color::Rgb(Rgb::new(0.09, 0.18, 0.38, None));
    let gold  = Color::Rgb(Rgb::new(0.78, 0.62, 0.18, None));
    let cream = Color::Rgb(Rgb::new(0.99, 0.97, 0.93, None));
    let white = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));
    let black = Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None));
    let gray  = Color::Rgb(Rgb::new(0.45, 0.45, 0.45, None));

    // ── 1. Cream background ───────────────────────────────────────────────────
    layer.set_fill_color(cream);
    layer.add_rect(Rect::new(Mm(0.0), Mm(0.0), Mm(297.0), Mm(210.0)));

    // ── 2. Navy header band (top 30mm: Y = 180–210) ───────────────────────────
    layer.set_fill_color(navy.clone());
    layer.add_rect(Rect::new(Mm(0.0), Mm(180.0), Mm(297.0), Mm(210.0)));

    // ── 3. Gold accent line under header ─────────────────────────────────────
    layer.set_outline_color(gold.clone());
    layer.set_outline_thickness(3.5);
    hline(&layer, 0.0, 297.0, 180.0);

    // ── 4. Navy footer band (bottom 20mm: Y = 0–20) ───────────────────────────
    layer.set_fill_color(navy.clone());
    layer.add_rect(Rect::new(Mm(0.0), Mm(0.0), Mm(297.0), Mm(20.0)));

    // ── 5. Gold accent line above footer ─────────────────────────────────────
    layer.set_outline_color(gold.clone());
    layer.set_outline_thickness(3.5);
    hline(&layer, 0.0, 297.0, 20.0);

    // ── 6. Gold vertical border lines (left & right) ─────────────────────────
    layer.set_outline_thickness(1.5);
    vline(&layer, 14.0, 20.0, 180.0);
    vline(&layer, 283.0, 20.0, 180.0);

    // ── 7. Title in header ────────────────────────────────────────────────────
    layer.set_fill_color(white.clone());
    let title = "CERTIFICATE OF COMPLETION";
    layer.use_text(title, 22.0, Mm(cx(title, 22.0)), Mm(193.0), &font_bold);

    // ── 8. Subtitle ───────────────────────────────────────────────────────────
    layer.set_fill_color(gray.clone());
    let subtitle = "This is to certify that";
    layer.use_text(subtitle, 11.0, Mm(cx(subtitle, 11.0)), Mm(168.0), &font_italic);

    // ── 9. Recipient name (large, navy) ───────────────────────────────────────
    layer.set_fill_color(navy.clone());
    let name = &data.recipient_name;
    layer.use_text(name, 26.0, Mm(cx(name, 26.0)), Mm(152.0), &font_bold);

    // ── 10. Gold underline below name ─────────────────────────────────────────
    let ul_w = (name.len() as f32 * 26.0 * 0.21).min(180.0_f32).max(40.0_f32);
    let ul_x = (148.5 - ul_w / 2.0).max(20.0_f32);
    layer.set_outline_color(gold.clone());
    layer.set_outline_thickness(1.2);
    hline(&layer, ul_x, ul_x + ul_w, 147.0);

    // ── 11. Achievement text ──────────────────────────────────────────────────
    layer.set_fill_color(black.clone());
    let desc1 = "has successfully completed the";
    layer.use_text(desc1, 11.0, Mm(cx(desc1, 11.0)), Mm(134.0), &font_regular);

    // Role name (larger, navy)
    layer.set_fill_color(navy.clone());
    let role_text = format!("{} Career Path", data.role_name);
    layer.use_text(&role_text, 15.0, Mm(cx(&role_text, 15.0)), Mm(121.0), &font_bold);

    // Module count (italic)
    layer.set_fill_color(gray.clone());
    let modules_text = format!(
        "Completing {} module(s) and final project with distinction",
        data.modules_count
    );
    layer.use_text(&modules_text, 9.5, Mm(cx(&modules_text, 9.5)), Mm(109.0), &font_italic);

    // ── 12. Gold separator — centered at X=148.5 ─────────────────────────────
    layer.set_outline_color(gold.clone());
    layer.set_outline_thickness(1.0);
    hline(&layer, 60.0, 237.0, 98.0); // width=177mm, centered: (60+237)/2 = 148.5 ✓

    // ── 13. Left column: date, code, verify (centered in X=20–222) ────────────
    // Right column X=230–280 is reserved for QR code.
    let left_cx = 121.0_f32;

    layer.set_fill_color(black.clone());
    let date_str = data.issued_at.format("%d %B %Y").to_string();
    let date_text = format!("Issued on  {}", date_str);
    layer.use_text(&date_text, 10.0, Mm(cx_at(&date_text, 10.0, left_cx)), Mm(84.0), &font_regular);

    let code_text = format!("Certificate Code:  {}", data.certificate_code);
    layer.use_text(&code_text, 10.0, Mm(cx_at(&code_text, 10.0, left_cx)), Mm(70.0), &font_bold);

    layer.set_fill_color(gray.clone());
    let verify_text = format!("Verify at: {}", data.verification_url);
    layer.use_text(&verify_text, 8.0, Mm(cx_at(&verify_text, 8.0, left_cx)), Mm(58.0), &font_italic);

    // ── 14. QR code (right column, X=232–272, Y=27–67) ───────────────────────
    // Natural size at ~244 DPI ≈ 20.8mm; scale 1.9 → ~39.5mm — clearly visible.
    if !data.qr_code_png.is_empty() {
        if let Ok(dyn_img) = printpdf::image_crate::load_from_memory(&data.qr_code_png) {
            let qr_image = printpdf::Image::from_dynamic_image(&dyn_img);
            qr_image.add_to_layer(
                layer.clone(),
                ImageTransform {
                    translate_x: Some(Mm(233.0)),
                    translate_y: Some(Mm(27.0)),
                    scale_x: Some(1.9),
                    scale_y: Some(1.9),
                    ..Default::default()
                },
            );
        }
    }

    // ── 15. Footer text ───────────────────────────────────────────────────────
    layer.set_fill_color(white.clone());
    let footer = "Scan the QR code or visit the verification URL to confirm authenticity";
    layer.use_text(footer, 7.5, Mm(cx(footer, 7.5)), Mm(8.0), &font_italic);

    // ── Render to bytes ───────────────────────────────────────────────────────
    let mut buf = std::io::BufWriter::new(Vec::new());
    doc.save(&mut buf)
        .map_err(|e| CertificateError::PdfGeneration(e.to_string()))?;
    buf.into_inner()
        .map_err(|e| CertificateError::PdfGeneration(e.to_string()))
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn hline(layer: &PdfLayerReference, x1: f32, x2: f32, y: f32) {
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(x1), Mm(y)), false),
            (Point::new(Mm(x2), Mm(y)), false),
        ],
        is_closed: false,
    });
}

fn vline(layer: &PdfLayerReference, x: f32, y1: f32, y2: f32) {
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(x), Mm(y1)), false),
            (Point::new(Mm(x), Mm(y2)), false),
        ],
        is_closed: false,
    });
}

// Center text at the page midpoint (X = 148.5mm)
fn cx(text: &str, font_pt: f32) -> f32 {
    cx_at(text, font_pt, 148.5)
}

// Center text at an arbitrary X midpoint
fn cx_at(text: &str, font_pt: f32, mid_x: f32) -> f32 {
    // Helvetica average char width ≈ 0.21mm per pt × char (empirically tuned)
    let w = text.len() as f32 * font_pt * 0.21;
    (mid_x - w / 2.0).max(15.0)
}
