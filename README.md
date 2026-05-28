# Career Path BE

REST API untuk aplikasi Career Path Recommendation — Rust + Axum + PostgreSQL.

## API Documentation

Dokumentasi interaktif tersedia setelah server berjalan:

| URL | Keterangan |
|---|---|
| [`http://localhost:3002/docs`](http://localhost:3002/docs) | **Scalar UI** |
| [`http://localhost:3002/swagger`](http://localhost:3002/swagger) | **Swagger UI** |
| [`http://localhost:3002/api-docs/openapi.json`](http://localhost:3002/api-docs/openapi.json) | **Raw OpenAPI 3.x spec** — untuk import Postman / codegen |

### Autentikasi di Swagger UI

1. Buka `http://localhost:3002/swagger`
2. Klik **Authorize** 🔒 di kanan atas
3. Login dulu via `POST /api/auth/login` untuk mendapatkan `access_token`
4. Masukkan token di field **bearer_auth** (tanpa prefix `Bearer`)
5. Klik **Authorize** → semua endpoint protected otomatis terisi header JWT

### Import ke Postman

- Postman → **Import** → masukkan URL: `http://localhost:3002/api-docs/openapi.json`
- Postman akan auto-generate collection lengkap dengan semua endpoint

### Generate TypeScript Client (opsional)

```bash
npx @openapitools/openapi-generator-cli generate \
  -i http://localhost:3002/api-docs/openapi.json \
  -g typescript-axios \
  -o ./generated-client
```

---

## Setup & Menjalankan dengan Docker

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) & Docker Compose v2
- Akun [Neon](https://neon.tech) (database PostgreSQL cloud — gratis)

### Quick Start

```bash
# 1. Clone repo
git clone <repo-url>
cd career-path-be

# 2. Buat file .env
cp .env.example .env
# Edit .env: isi DATABASE_URL dari Neon, dan JWT_SECRET

# 3. Jalankan migrasi ke Neon (sekali saja, atau setiap ada migration baru)
sqlx migrate run

# 4. Build & jalankan
docker compose up --build
```

Akses:
- **API**: `http://localhost:3002`
- **Health check**: `http://localhost:3002/health`
- **Scalar UI**: `http://localhost:3002/docs`
- **Swagger UI**: `http://localhost:3002/swagger`

### Install sqlx-cli (untuk migrate)

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls
```

### Stop & Start ulang

```bash
# Stop container
docker compose down

# Jalankan lagi (tanpa rebuild)
docker compose up -d

# Jalankan lagi + rebuild (setelah ada perubahan code)
docker compose up --build -d
```

---

### Docker Files Overview

| File | Fungsi |
|---|---|
| `Dockerfile` | Multi-stage build pakai `cargo-chef` — image akhir ~120 MB |
| `.dockerignore` | Exclude `target/`, `.env`, `storage/` dari build context |
| `docker-compose.yml` | App service + volume storage |
| `docker-compose.override.yml` | (kosong — untuk override lokal jika diperlukan) |
| `docker-compose.prod.yml` | (kosong — untuk override production jika diperlukan) |

---

### Environment Variables

Salin `.env.example` ke `.env` dan isi variabel berikut:

| Variabel | Keterangan |
|---|---|
| `DATABASE_URL` | Connection string Neon (dari dashboard neon.tech) |
| `JWT_SECRET` | String acak ≥ 32 karakter (`openssl rand -base64 48`) |
| `JWT_EXPIRES_IN` | TTL access token dalam detik (default: 900 = 15 menit) |
| `REFRESH_TOKEN_EXPIRES_IN` | TTL refresh token dalam detik (default: 604800 = 7 hari) |
| `SERVER_PORT` | Port server (default: 3002) |
| `RUST_LOG` | Log level (default: `info,career_path_be=debug`) |
| `APP_BASE_URL` | Base URL publik server (default: `http://localhost:3002`) — digunakan untuk verification URL di sertifikat |

---

### Troubleshooting

**Build Docker lama pertama kali** — cargo-chef butuh ~10-20 menit compile dependencies. Build berikutnya incremental < 2 menit (layer ter-cache).

**SQLx offline error saat Docker build**:
```
error: set `SQLX_OFFLINE=true` to run without database connection
```
Solusi: jalankan `cargo sqlx prepare` lalu commit folder `.sqlx/`.

**TLS error saat sqlx migrate**:
```
TLS upgrade required but SQLx was built without TLS support
```
Solusi: install ulang sqlx-cli dengan flag rustls:
```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls --force
```

---

## Default Admin Credentials

Setelah menjalankan migration, akan ada satu akun admin seeder:

| Field | Value |
|---|---|
| Email | `admin@careerpath.local` |
| Password | `ChangeMeASAP123!` |
| UUID | `00000000-0000-0000-0000-000000000001` |

> **WAJIB GANTI PASSWORD SETELAH FIRST LOGIN.**

Untuk generate hash password baru:
```bash
cargo run --bin gen_password_hash -- "PasswordBaruKamu123!"
```
Kemudian update via:
```sql
UPDATE users SET password_hash = '...' WHERE id = '00000000-0000-0000-0000-000000000001';
```

---

## Endpoints

> Semua endpoint yang butuh login menggunakan header `Authorization: Bearer <token>`.

### Auth (`/api/auth`)

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `POST` | `/api/auth/register` | ✗ | Daftar akun baru |
| `POST` | `/api/auth/login` | ✗ | Login, dapat access + refresh token |
| `POST` | `/api/auth/refresh` | ✗ | Perbarui access token pakai refresh token |
| `POST` | `/api/auth/logout` | ✓ | Logout, invalidate refresh token |
| `GET` | `/api/auth/me` | ✓ | Profil user yang sedang login |

---

### Pre-Quiz — Role Determining (`/api/quiz`)

Digunakan user untuk menentukan career role sebelum mulai belajar.

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `GET` | `/api/quiz/questions` | ✓ | Ambil semua soal pre-quiz |
| `POST` | `/api/quiz/attempts` | ✓ | Mulai sesi attempt baru |
| `POST` | `/api/quiz/attempts/:id/answers` | ✓ | Simpan jawaban per soal |
| `POST` | `/api/quiz/attempts/:id/submit` | ✓ | Submit attempt, dapatkan hasil role |
| `GET` | `/api/quiz/attempts/:id/result` | ✓ | Lihat hasil attempt |
| `GET` | `/api/quiz/history` | ✓ | Riwayat semua attempt user |

---

### Learning (`/api/learning`)

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `GET` | `/api/learning/modules` | ✓ | Daftar modul sesuai role user |
| `GET` | `/api/learning/modules/:id` | ✓ | Detail modul |
| `GET` | `/api/learning/modules/:id/quiz` | ✓ | Soal final quiz modul |
| `POST` | `/api/learning/modules/:id/quiz/submit` | ✓ | Submit jawaban final quiz |
| `GET` | `/api/learning/modules/:id/quiz/history` | ✓ | Riwayat attempt final quiz |
| `GET` | `/api/learning/submaterials/:id` | ✓ | Detail submaterial |
| `POST` | `/api/learning/submaterials/:id/complete` | ✓ | Tandai submaterial selesai dibaca |
| `GET` | `/api/learning/submaterials/:id/quiz` | ✓ | Soal mini quiz submaterial |
| `POST` | `/api/learning/submaterials/:id/quiz/submit` | ✓ | Submit jawaban mini quiz |
| `GET` | `/api/learning/progress` | ✓ | Progress belajar user (per modul & submaterial) |

---

### Projects (`/api/projects`)

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `GET` | `/api/projects/me` | ✓ | Project milik user (sesuai role) |
| `GET` | `/api/projects/:id` | ✓ | Detail project |
| `POST` | `/api/projects/:id/submit` | ✓ | Submit project (upload ZIP, maks 25 MB) |
| `GET` | `/api/projects/:id/submissions` | ✓ | Daftar submission user untuk project ini |
| `GET` | `/api/projects/submissions/:submission_id/download` | ✓ | Download file ZIP submission |

---

### Dashboard (`/api/dashboard`)

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `GET` | `/api/dashboard` | ✓ | Ringkasan progress, role, dan next action |
| `GET` | `/api/dashboard/learning-summary` | ✓ | Ringkasan modul & submaterial yang sudah selesai |
| `GET` | `/api/dashboard/activity` | ✓ | Log aktivitas terbaru user |

---

### Admin — User Management (`/api/admin`)

> Semua endpoint `/api/admin/*` memerlukan Bearer token dengan role `admin`.

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/users` | Daftar user (paginasi, filter, sort) |
| `GET` | `/api/admin/users/:id` | Detail user + quiz/module/project history |
| `PATCH` | `/api/admin/users/:id` | Update nama dan/atau role |
| `POST` | `/api/admin/users/:id/deactivate` | Soft delete dengan alasan |
| `POST` | `/api/admin/users/:id/activate` | Reaktivasi user |
| `GET` | `/api/admin/users/:id/audit-logs` | Audit trail untuk user tertentu |
| `GET` | `/api/admin/audit-logs` | Audit trail global |

---

### Admin — Content Management (`/api/admin`)

#### Roles

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/roles` | Daftar career role |
| `POST` | `/api/admin/roles` | Buat role baru |
| `GET` | `/api/admin/roles/:id` | Detail role + stats |
| `PATCH` | `/api/admin/roles/:id` | Update nama/deskripsi role |
| `POST` | `/api/admin/roles/:id/deactivate` | Non-aktifkan role |
| `POST` | `/api/admin/roles/:id/restore` | Aktifkan kembali role |

#### Learning Modules

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/modules` | Daftar modul |
| `POST` | `/api/admin/modules` | Buat modul baru |
| `GET` | `/api/admin/modules/:id` | Detail modul + stats |
| `PATCH` | `/api/admin/modules/:id` | Update modul |
| `DELETE` | `/api/admin/modules/:id` | Soft delete. `?force=true` → hard delete |
| `POST` | `/api/admin/modules/:id/restore` | Publikasikan kembali modul |
| `GET` | `/api/admin/modules/:module_id/final-quiz` | Daftar soal final quiz modul |

#### Submaterials

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/submaterials` | Daftar submaterial |
| `POST` | `/api/admin/submaterials` | Buat submaterial |
| `GET` | `/api/admin/submaterials/:id` | Detail submaterial |
| `PATCH` | `/api/admin/submaterials/:id` | Update submaterial |
| `DELETE` | `/api/admin/submaterials/:id` | Soft delete. `?force=true` → hard delete |
| `POST` | `/api/admin/submaterials/:id/restore` | Publish kembali |
| `GET` | `/api/admin/submaterials/:submaterial_id/quiz` | Daftar soal mini quiz |

#### Mini Quiz Questions (per Submaterial)

| Method | Path | Deskripsi |
|---|---|---|
| `POST` | `/api/admin/submaterial-quiz-questions` | Tambah soal mini quiz |
| `PATCH` | `/api/admin/submaterial-quiz-questions/:id` | Update soal |
| `PATCH` | `/api/admin/submaterial-quiz-questions/:id/options` | Ganti semua opsi |
| `DELETE` | `/api/admin/submaterial-quiz-questions/:id` | Hapus soal |

#### Final Quiz Questions (per Module)

| Method | Path | Deskripsi |
|---|---|---|
| `POST` | `/api/admin/module-quiz-questions` | Tambah soal final quiz |
| `PATCH` | `/api/admin/module-quiz-questions/:id` | Update soal |
| `PATCH` | `/api/admin/module-quiz-questions/:id/options` | Ganti semua opsi |
| `DELETE` | `/api/admin/module-quiz-questions/:id` | Hapus soal |

#### Pre-Quiz Questions (Role Determining)

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/pre-quiz-questions` | Daftar soal pre-quiz |
| `POST` | `/api/admin/pre-quiz-questions` | Tambah soal pre-quiz |
| `GET` | `/api/admin/pre-quiz-questions/:id` | Detail soal |
| `PATCH` | `/api/admin/pre-quiz-questions/:id` | Update soal |
| `PATCH` | `/api/admin/pre-quiz-questions/:id/options` | Ganti semua opsi |
| `POST` | `/api/admin/pre-quiz-questions/:id/deactivate` | Non-aktifkan soal |
| `POST` | `/api/admin/pre-quiz-questions/:id/restore` | Aktifkan kembali |

#### Projects

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/projects` | Daftar project |
| `POST` | `/api/admin/projects` | Buat project |
| `GET` | `/api/admin/projects/:id` | Detail project |
| `PATCH` | `/api/admin/projects/:id` | Update project |
| `POST` | `/api/admin/projects/:id/unpublish` | Unpublish project |
| `POST` | `/api/admin/projects/:id/restore` | Publish kembali |

---

### Admin — Submission Review (`/api/admin`)

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/submissions/queue/stats` | Statistik antrian (pending, oldest, avg time) |
| `GET` | `/api/admin/submissions` | Daftar submission (filter + paginasi) |
| `GET` | `/api/admin/submissions/:id` | Detail submission + riwayat review |
| `GET` | `/api/admin/submissions/:id/download` | Download file ZIP submission |
| `POST` | `/api/admin/submissions/:id/approve` | Setujui submission — **otomatis issue sertifikat jika eligible** |
| `POST` | `/api/admin/submissions/:id/reject` | Tolak submission |

---

### Certificate (`/api/certificates`, `/api/verify`)

Sertifikat diterbitkan **otomatis** saat admin menyetujui final project, jika user sudah memenuhi semua syarat eligibility.

**Syarat Eligibility:**
- ✅ User sudah menyelesaikan pre-quiz (memiliki role)
- ✅ Semua modul role sudah lulus final quiz (completed == total)
- ✅ Ada satu project submission dengan status `approved`

#### User Endpoints

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `GET` | `/api/certificates/me` | ✓ | Daftar sertifikat milik user |
| `GET` | `/api/certificates/me/:id` | ✓ | Detail sertifikat + modul selesai + QR code |
| `GET` | `/api/certificates/me/:id/download.pdf` | ✓ | Download sertifikat PDF (A4 landscape) |

#### Public Endpoint (tanpa login)

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `GET` | `/api/verify/:code` | ✗ | Verifikasi keaslian sertifikat via kode |

Response verifikasi:

```json
{
  "success": true,
  "data": {
    "is_valid": true,
    "status": "valid",
    "message": "Certificate valid",
    "certificate": {
      "certificate_code": "CERT-2026-XK7M3PQR",
      "recipient_name": "Budi Santoso",
      "role_name": "Frontend Developer",
      "issued_at": "2026-05-28T10:00:00Z",
      "modules_completed_count": 5
    }
  }
}
```

`status` bisa: `"valid"` | `"revoked"` | `"not_found"`

#### Admin Endpoints

> Memerlukan Bearer token dengan role `admin`.

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/certificates` | Daftar semua sertifikat (filter + paginasi) |
| `POST` | `/api/admin/certificates/:id/revoke` | Cabut sertifikat dengan alasan |
| `POST` | `/api/admin/certificates/:id/restore` | Pulihkan sertifikat yang dicabut |

**Filter `GET /api/admin/certificates`:**

| Query param | Tipe | Deskripsi |
|---|---|---|
| `page` | `int` | Halaman (default: 1) |
| `per_page` | `int` | Item per halaman (default: 20, maks: 100) |
| `user_id` | `uuid` | Filter by user |
| `role_id` | `uuid` | Filter by role |
| `is_revoked` | `bool` | Filter sertifikat aktif / dicabut |
| `from_date` | `timestamptz` | Tanggal terbit mulai |
| `to_date` | `timestamptz` | Tanggal terbit sampai |
| `search` | `string` | Cari by `certificate_code` atau `recipient_name` |

**Response `POST .../approve` saat sertifikat diterbitkan:**

```json
{
  "submission_id": "...",
  "status": "approved",
  "certificate_issued": true,
  "certificate_id": "uuid...",
  "certificate_code": "CERT-2026-XK7M3PQR",
  "user": { ... },
  "project": { ... }
}
```

---

## Tech Stack

| Layer | Library |
|---|---|
| HTTP | axum 0.7 |
| Async runtime | tokio |
| Database | PostgreSQL via sqlx 0.8 (hosted on Neon) |
| Auth | JWT (jsonwebtoken) + Argon2 |
| Validation | validator |
| Sanitization | ammonia (HTML strip) |
| Error handling | thiserror + anyhow |
| Logging | tracing + tracing-subscriber |
| API Docs | utoipa 5 + Scalar UI + Swagger UI |
| PDF Generation | printpdf 0.7 (built-in Helvetica, QR embed) |
| QR Code | qrcode 0.14 + image 0.25 (PNG encoder) |

---

## Struktur Folder

```
src/
├── main.rs              # bootstrap
├── lib.rs               # expose modules
├── app.rs               # Router utama + middleware
├── config.rs            # env config
├── state.rs             # AppState (db pool + config)
├── error/mod.rs         # AppError + IntoResponse
├── middleware/
│   ├── auth.rs          # AuthUser extractor (JWT)
│   └── role_guard.rs    # AdminUser extractor
├── shared/
│   ├── jwt.rs              # create/verify token
│   ├── password.rs         # argon2 hash/verify
│   ├── pagination.rs       # PaginationQuery + PaginatedResponse
│   ├── sanitization.rs     # sanitize_plain_text
│   ├── certificate_code.rs # generate "CERT-YYYY-XXXXXXXX"
│   └── response.rs         # ApiResponse<T> wrapper
├── db/pool.rs           # PgPool setup
└── features/
    ├── auth/            # register · login · refresh · logout · me
    ├── user/            # profil user
    ├── quiz/            # role quiz
    ├── learning/        # modules · submaterials · mini quiz · final quiz
    ├── project/         # submit ZIP · download · review status
    ├── dashboard/       # summary · recent activities · next action
    ├── certificate/     # auto-issue · PDF · QR · public verify · admin revoke
    │   ├── entity.rs    # Certificate struct (DB row)
    │   ├── dto.rs       # Request/response DTOs
    │   ├── error.rs     # CertificateError → AppError
    │   ├── eligibility.rs  # check_eligibility() — pure function, unit-tested
    │   ├── qr.rs        # generate_qr_png / generate_qr_data_url
    │   ├── pdf.rs       # generate_pdf() → Vec<u8> (A4 landscape)
    │   ├── repository.rs
    │   ├── service.rs   # try_issue (atomic) · CRUD
    │   ├── handler.rs
    │   └── routes.rs
    └── admin/
        ├── audit/       # audit trail
        ├── user/        # CRUD users
        ├── content/     # Content CRUD
        └── submission/  # Review queue · approve/reject · auto-issue cert
```
