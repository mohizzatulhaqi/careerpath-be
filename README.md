# Career Path BE

REST API untuk aplikasi Career Path Recommendation — Rust + Axum + PostgreSQL.

## Tech Stack

| Layer | Library |
|---|---|
| HTTP | axum 0.7 |
| Async runtime | tokio |
| Database | PostgreSQL via sqlx 0.8 |
| Auth | JWT (jsonwebtoken) + Argon2 |
| Validation | validator |
| Error handling | thiserror + anyhow |
| Logging | tracing + tracing-subscriber |

---

## Prasyarat

- Rust ≥ 1.75 (stable)
- PostgreSQL ≥ 14
- `sqlx-cli` (opsional, untuk migration)

Install sqlx-cli:
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

---

## Setup

### 1. Buat database

```bash
psql -U postgres -c "CREATE DATABASE career_path;"
```

### 2. Konfigurasi environment

```bash
cp .env.example .env
# Edit .env sesuai kredensial PostgreSQL kamu
```

Variabel yang diperlukan:

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/career_path
JWT_SECRET=<string acak panjang ≥ 32 karakter>
JWT_EXPIRES_IN=86400       # detik (default: 24 jam)
SERVER_PORT=3000
RUST_LOG=info,career_path_be=debug
```

### 3. Jalankan migration

**Opsi A — sqlx-cli (recommended):**
```bash
sqlx migrate run
```

**Opsi B — psql manual:**
```bash
psql "$DATABASE_URL" -f migrations/20240101000000_create_users.sql
```

### 4. Build & run

```bash
cargo run
```

Server berjalan di `http://localhost:3000`.

---

## ⚠️ Default Admin Credentials

Setelah menjalankan migration, akan ada satu akun admin seeder:

| Field | Value |
|---|---|
| Email | `admin@careerpath.local` |
| Password | `ChangeMeASAP123!` |
| UUID | `00000000-0000-0000-0000-000000000001` |

> **WAJIB GANTI PASSWORD SETELAH FIRST LOGIN.**  
> Hash ini adalah argon2id publik yang diketahui. Biarkan password ini aktif di production = risiko keamanan kritis.

Untuk generate hash password baru:
```bash
cargo run --bin gen_password_hash -- "PasswordBaruKamu123!"
```
Kemudian update via `UPDATE users SET password_hash = '...' WHERE id = '00000000-0000-0000-0000-000000000001';`

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
| `GET` | `/api/admin/audit-logs` | Audit trail global (filter: action, admin_id, target_type, dll) |

**Query params `GET /api/admin/users`:**

| Param | Type | Deskripsi |
|---|---|---|
| `page` | int | Halaman (default: 1) |
| `per_page` | int | Per halaman (default: 20, max: 100) |
| `search` | string | Cari berdasarkan nama atau email (ILIKE) |
| `role` | string | Filter by role (`user` / `admin`) |
| `is_active` | bool | Filter by status aktif |
| `sort` | string | Kolom sort: `name`, `email`, `role`, `created_at` |
| `order` | string | Arah sort: `asc` / `desc` (default: `desc`) |

---

### Admin — Content Management (role: admin)

Semua endpoint di bawah memerlukan Bearer token dengan role `admin`. Prinsip:
- **Soft delete default** — resource di-unpublish/deactivate, bukan dihapus permanen.
- **Force flag (`?force=true`)** — diperlukan untuk operasi yang punya side effect (menghapus resource yang dipakai user, mengubah opsi quiz yang sudah dijawab, dll.). Tanpa `force`, server mengembalikan `409 REQUIRES_FORCE` beserta `affected_count`.
- **Audit trail** — semua mutasi dicatat ke `admin_audit_logs`.

#### Roles

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/roles` | Daftar career role (filter: is_active, page, per_page) |
| `POST` | `/api/admin/roles` | Buat role baru (code harus lowercase alphanumeric + `_`) |
| `GET` | `/api/admin/roles/:id` | Detail role + stats (total modules, total users) |
| `PATCH` | `/api/admin/roles/:id` | Update nama/deskripsi role |
| `POST` | `/api/admin/roles/:id/deactivate` | Non-aktifkan role |
| `POST` | `/api/admin/roles/:id/restore` | Aktifkan kembali role |

#### Learning Modules

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/modules` | Daftar modul (filter: role_id, is_published, page, per_page) |
| `POST` | `/api/admin/modules` | Buat modul baru (wajib role_id aktif, order_index otomatis) |
| `GET` | `/api/admin/modules/:id` | Detail modul + stats |
| `PATCH` | `/api/admin/modules/:id` | Update modul (order conflict → 409) |
| `DELETE` | `/api/admin/modules/:id` | Soft delete (unpublish). `?force=true` → hard delete jika ada user progress |
| `POST` | `/api/admin/modules/:id/restore` | Publikasikan kembali modul |
| `GET` | `/api/admin/modules/:module_id/final-quiz` | Daftar soal final quiz modul |

#### Submaterials

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/submaterials` | Daftar submaterial (filter: module_id, is_published, page, per_page) |
| `POST` | `/api/admin/submaterials` | Buat submaterial (respons menyertakan `requires_mini_quiz: true`) |
| `GET` | `/api/admin/submaterials/:id` | Detail submaterial |
| `PATCH` | `/api/admin/submaterials/:id` | Update submaterial |
| `DELETE` | `/api/admin/submaterials/:id` | Soft delete. `?force=true` → hard delete |
| `POST` | `/api/admin/submaterials/:id/restore` | Publish kembali |
| `GET` | `/api/admin/submaterials/:submaterial_id/quiz` | Daftar soal mini quiz |

#### Mini Quiz Questions (per Submaterial)

| Method | Path | Deskripsi |
|---|---|---|
| `POST` | `/api/admin/submaterial-quiz-questions` | Tambah soal (wajib tepat 1 opsi `is_correct=true`) |
| `PATCH` | `/api/admin/submaterial-quiz-questions/:id` | Update teks/order soal |
| `PATCH` | `/api/admin/submaterial-quiz-questions/:id/options` | Ganti semua opsi. `?force=true` jika sudah ada attempt |
| `DELETE` | `/api/admin/submaterial-quiz-questions/:id` | Soft delete. `?force=true` jika ada attempt |

#### Final Quiz Questions (per Module)

| Method | Path | Deskripsi |
|---|---|---|
| `POST` | `/api/admin/module-quiz-questions` | Tambah soal (minimal 1 opsi `is_correct=true`, boleh lebih dari 1) |
| `PATCH` | `/api/admin/module-quiz-questions/:id` | Update soal |
| `PATCH` | `/api/admin/module-quiz-questions/:id/options` | Ganti semua opsi. `?force=true` jika ada attempt |
| `DELETE` | `/api/admin/module-quiz-questions/:id` | Soft delete. `?force=true` jika ada attempt |

#### Pre-Quiz Questions (Role Determining)

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/pre-quiz-questions` | Daftar soal pre-quiz (filter: is_active, page, per_page) |
| `POST` | `/api/admin/pre-quiz-questions` | Tambah soal + opsi + bobot per role |
| `GET` | `/api/admin/pre-quiz-questions/:id` | Detail soal + opsi + bobot |
| `PATCH` | `/api/admin/pre-quiz-questions/:id` | Update teks/order/is_active soal |
| `PATCH` | `/api/admin/pre-quiz-questions/:id/options` | Ganti semua opsi + bobot. `?force=true` jika ada attempt |
| `POST` | `/api/admin/pre-quiz-questions/:id/deactivate` | Non-aktifkan soal |
| `POST` | `/api/admin/pre-quiz-questions/:id/restore` | Aktifkan kembali soal |

#### Projects

| Method | Path | Deskripsi |
|---|---|---|
| `GET` | `/api/admin/projects` | Daftar project (filter: role_id, is_published, page, per_page) |
| `POST` | `/api/admin/projects` | Buat project (1 project per role) |
| `GET` | `/api/admin/projects/:id` | Detail project + jumlah submission |
| `PATCH` | `/api/admin/projects/:id` | Update project |
| `POST` | `/api/admin/projects/:id/unpublish` | Unpublish project |
| `POST` | `/api/admin/projects/:id/restore` | Publish kembali |

---

## Test dengan curl

### Register

```bash
curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret123","name":"Budi"}' | jq
```

Response `201`:
```json
{
  "success": true,
  "data": {
    "token": "<jwt>",
    "user": { "id": "...", "email": "user@example.com", "name": "Budi", "role": "user", "created_at": "..." }
  }
}
```

### Login

```bash
curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret123"}' | jq
```

### Me (protected)

```bash
TOKEN="<token dari login>"

curl -s http://localhost:3000/api/auth/me \
  -H "Authorization: Bearer $TOKEN" | jq
```

### Error response format

```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid or expired token"
  }
}
```

### Admin — Module lifecycle (curl)

```bash
ADMIN_TOKEN="<token admin>"

# 1. Buat career role dulu (butuh role sebelum buat modul)
ROLE_ID=$(curl -s -X POST http://localhost:3000/api/admin/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"code":"backend_dev","name":"Backend Developer","description":"Jalur karir backend"}' \
  | jq -r '.data.id')

# 2. Daftar modul
curl -s "http://localhost:3000/api/admin/modules?role_id=$ROLE_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" | jq

# 3. Buat modul baru
MODULE_ID=$(curl -s -X POST http://localhost:3000/api/admin/modules \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"role_id\":\"$ROLE_ID\",\"title\":\"Dasar HTTP\",\"description\":\"HTTP fundamentals\"}" \
  | jq -r '.data.id')

# 4. Update modul
curl -s -X PATCH "http://localhost:3000/api/admin/modules/$MODULE_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Dasar HTTP & REST","description":"HTTP + REST API fundamentals"}' | jq

# 5. Unpublish (soft delete) — tidak perlu force jika belum ada user progress
curl -s -X DELETE "http://localhost:3000/api/admin/modules/$MODULE_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" | jq
# → 200 {message: "Module unpublished"}

# 5b. Jika ada user yang sudah pernah quiz di modul ini, server akan menolak:
# → 409 {code: "REQUIRES_FORCE", affected_count: 5}
# Gunakan ?force=true untuk hard delete:
curl -s -X DELETE "http://localhost:3000/api/admin/modules/$MODULE_ID?force=true" \
  -H "Authorization: Bearer $ADMIN_TOKEN" | jq
# → 200 {message: "Module deleted permanently"}

# 6. Restore (publish kembali) — jika belum dihapus permanen
curl -s -X POST "http://localhost:3000/api/admin/modules/$MODULE_ID/restore" \
  -H "Authorization: Bearer $ADMIN_TOKEN" | jq

# 7. Force flag — response 409 REQUIRES_FORCE example:
# {
#   "success": false,
#   "error": {
#     "code": "REQUIRES_FORCE",
#     "message": "Module has 3 users with progress. Add ?force=true to delete permanently.",
#     "affected_count": 3
#   }
# }
```

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
│   └── response.rs      # ApiResponse<T> wrapper
├── db/pool.rs           # PgPool setup
└── features/
    ├── auth/            # register · login · refresh · logout · me
    ├── user/            # profil user (get/update)
    ├── quiz/            # role quiz (questions · submit · result)
    ├── learning/        # modules · submaterials · mini quiz · final quiz · gating
    ├── project/         # submit ZIP · download · review status
    ├── dashboard/       # summary · recent activities · next action
    └── admin/
        ├── audit/       # audit trail (entity · dto · repository · service)
        ├── user/        # CRUD users · deactivate · activate · audit logs
        └── content/     # Content CRUD (role · module · submaterial · mini/final/pre quiz · project)
```

Setiap feature mengikuti pola:
```
feature/
├── mod.rs
├── routes.rs      → Router definition only
├── handler.rs     → HTTP layer (extract → call service → return)
├── service.rs     → business logic
├── repository.rs  → DB queries only
├── dto.rs         → request/response structs + validation
├── entity.rs      → DB row structs (FromRow)
└── error.rs       → feature error → AppError conversion
```

---

## Compile-time query checking (opsional)

Untuk mengaktifkan `sqlx::query_as!` macro dengan compile-time verification:

```bash
# Pastikan DB sudah up dan migration sudah dijalankan
cargo sqlx prepare
```

Ini akan membuat folder `.sqlx/` yang bisa di-commit agar CI bisa build tanpa DB live (`SQLX_OFFLINE=true`).
