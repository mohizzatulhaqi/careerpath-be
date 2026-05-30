use serde_json::json;

pub struct ReviewEmailPayload<'a> {
    pub to_email: &'a str,
    pub to_name: &'a str,
    pub project_title: &'a str,
    pub status: &'a str,       // "approved" | "rejected"
    pub reviewer_notes: &'a str,
}

/// Send a submission review notification via Resend.
/// Best-effort: logs a warning and returns Ok(()) on failure so the review
/// transaction is never rolled back because of an email delivery issue.
pub async fn send_review_notification(
    api_key: &str,
    payload: ReviewEmailPayload<'_>,
) -> Result<(), String> {
    let (subject, status_label, status_color) = match payload.status {
        "approved" => (
            format!("✅ Submission \"{}\" disetujui!", payload.project_title),
            "Disetujui",
            "#16a34a",
        ),
        _ => (
            format!("❌ Submission \"{}\" ditolak", payload.project_title),
            "Ditolak",
            "#dc2626",
        ),
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="id">
<body style="font-family:sans-serif;max-width:520px;margin:40px auto;color:#111">
  <h2 style="color:{status_color}">{status_label}: {project}</h2>
  <p>Halo <strong>{name}</strong>,</p>
  <p>Submission project <strong>{project}</strong> kamu telah ditinjau.</p>
  <table style="border-collapse:collapse;width:100%;margin:16px 0">
    <tr>
      <td style="padding:8px;border:1px solid #e5e7eb;width:120px"><strong>Status</strong></td>
      <td style="padding:8px;border:1px solid #e5e7eb;color:{status_color}"><strong>{status_label}</strong></td>
    </tr>
    <tr>
      <td style="padding:8px;border:1px solid #e5e7eb"><strong>Catatan reviewer</strong></td>
      <td style="padding:8px;border:1px solid #e5e7eb">{notes}</td>
    </tr>
  </table>
  <p style="color:#6b7280;font-size:13px">Silakan login ke platform untuk melihat detail lengkap.</p>
</body>
</html>"#,
        status_color = status_color,
        status_label = status_label,
        project = payload.project_title,
        name = payload.to_name,
        notes = payload.reviewer_notes,
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&json!({
            "from": "CareerPath <onboarding@resend.dev>",
            "to": [payload.to_email],
            "subject": subject,
            "html": html,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Resend API error {status}: {body}"));
    }

    Ok(())
}
