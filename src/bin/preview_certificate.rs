use career_path_be::features::certificate::{pdf::{generate_pdf, CertificatePdfData}, qr};
use chrono::Utc;
use std::fs;

fn main() {
    let verification_url = "http://localhost:3002/verify/CERT-2026-XK7M3PQR";

    let qr_png = qr::generate_qr_png(verification_url).expect("QR gen failed");

    let data = CertificatePdfData {
        certificate_code: "CERT-2026-XK7M3PQR".to_string(),
        recipient_name: "Budi Santoso".to_string(),
        role_name: "Frontend Developer".to_string(),
        issued_at: Utc::now(),
        modules_count: 5,
        verification_url: verification_url.to_string(),
        qr_code_png: qr_png,
    };

    let pdf_bytes = generate_pdf(&data).expect("PDF gen failed");
    let out_path = "/tmp/sample_certificate.pdf";
    fs::write(out_path, &pdf_bytes).expect("write failed");
    println!("PDF saved to {out_path}");
}
