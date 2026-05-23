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

### Auth

| Method | Path | Auth | Deskripsi |
|---|---|---|---|
| `POST` | `/api/auth/register` | ✗ | Daftar akun baru |
| `POST` | `/api/auth/login` | ✗ | Login, dapat JWT |
| `GET` | `/api/auth/me` | ✓ Bearer | Profil user yang login |

### Admin — User Management (role: admin)

Semua endpoint `/api/admin/*` memerlukan Bearer token dengan role `admin`.

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

### Admin — Content Management (role: admin)

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

#### Submaterials

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/submaterials` | Daftar submaterial |
| `POST` | `/api/admin/submaterials` | Buat submaterial |
| `GET` | `/api/admin/submaterials/:id` | Detail submaterial |
| `PATCH` | `/api/admin/submaterials/:id` | Update submaterial |
| `DELETE` | `/api/admin/submaterials/:id` | Soft delete. `?force=true` → hard delete |
| `POST` | `/api/admin/submaterials/:id/restore` | Publish kembali |

#### Quiz Questions

| Method | Path | Deskripsi |
|---|---|---|
| `POST` | `/api/admin/submaterial-quiz-questions` | Tambah soal mini quiz |
| `PATCH` | `/api/admin/submaterial-quiz-questions/:id` | Update soal |
| `PATCH` | `/api/admin/submaterial-quiz-questions/:id/options` | Ganti semua opsi |
| `DELETE` | `/api/admin/submaterial-quiz-questions/:id` | Hapus soal |
| `POST` | `/api/admin/module-quiz-questions` | Tambah soal final quiz |
| `PATCH` | `/api/admin/module-quiz-questions/:id` | Update soal |
| `PATCH` | `/api/admin/module-quiz-questions/:id/options` | Ganti semua opsi |
| `DELETE` | `/api/admin/module-quiz-questions/:id` | Hapus soal |
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

### Admin — Submission Review (role: admin)

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/submissions` | Daftar submission |
| `GET` | `/api/admin/submissions/:id` | Detail submission + riwayat review |
| `GET` | `/api/admin/submissions/:id/download` | Download file ZIP |
| `POST` | `/api/admin/submissions/:id/approve` | Setujui submission |
| `POST` | `/api/admin/submissions/:id/reject` | Tolak submission |
| `GET` | `/api/admin/submissions/queue/stats` | Statistik antrian |

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
│   ├── jwt.rs           # create/verify token
│   ├── password.rs      # argon2 hash/verify
│   ├── pagination.rs    # PaginationQuery + PaginatedResponse
│   ├── sanitization.rs  # sanitize_plain_text
│   └── response.rs      # ApiResponse<T> wrapper
├── db/pool.rs           # PgPool setup
└── features/
    ├── auth/            # register · login · refresh · logout · me
    ├── user/            # profil user
    ├── quiz/            # role quiz
    ├── learning/        # modules · submaterials · mini quiz · final quiz
    ├── project/         # submit ZIP · download · review status
    ├── dashboard/       # summary · recent activities · next action
    └── admin/
        ├── audit/       # audit trail
        ├── user/        # CRUD users
        ├── content/     # Content CRUD
        └── submission/  # Review queue · approve/reject
```
