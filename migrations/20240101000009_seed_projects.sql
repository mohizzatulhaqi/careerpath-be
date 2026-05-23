-- Seed one final project per role
-- Role IDs match the seed in 20240101000000_create_users.sql:
--   frontend     = 00000001-0000-0000-0000-000000000001
--   backend      = 00000001-0000-0000-0000-000000000002
--   uiux         = 00000001-0000-0000-0000-000000000003
--   data_analyst = 00000001-0000-0000-0000-000000000004
--   mobile       = 00000001-0000-0000-0000-000000000005

INSERT INTO projects (role_id, title, description, requirements, estimated_hours) VALUES
(
    '00000001-0000-0000-0000-000000000001',
    'Capstone Project: Web App Lengkap',
    E'## Instruksi Final Project — Frontend Developer\n\nBangun sebuah **web application lengkap** menggunakan framework modern (React, Vue, atau Svelte). Aplikasi ini harus mendemonstrasikan pemahaman kamu terhadap seluruh materi yang sudah dipelajari.\n\n### Pilihan Tema\nPilih salah satu tema berikut:\n- **E-Commerce Mini**: Katalog produk, keranjang, checkout flow\n- **Dashboard Analytics**: Visualisasi data dengan chart, filter, dan tabel interaktif\n- **Todo App with Auth**: Task manager dengan autentikasi, kategori, dan prioritas\n- **Personal Blog**: CMS sederhana dengan editor markdown dan halaman publik\n\n### Persyaratan Teknis\n1. Gunakan **client-side routing** (React Router / Vue Router)\n2. Implementasi **state management** (Context API / Redux / Pinia / Zustand)\n3. Konsumsi **REST API** (boleh mock API atau backend sendiri)\n4. **Responsive design** — tampilan harus baik di mobile (≤768px) dan desktop\n5. Minimal **5 halaman/view** yang berbeda\n6. Gunakan **komponen reusable** dan arsitektur folder yang rapi\n\n### Format Submission\nUpload file **ZIP** yang berisi:\n- Source code lengkap (tanpa node_modules)\n- File `README.md` dengan instruksi setup dan menjalankan project\n- Screenshot tampilan aplikasi (minimal 3 halaman)\n- (Opsional) URL deploy jika sudah di-deploy',
    E'## Checklist Penilaian\n\n- [ ] Minimal 5 halaman/view dengan routing yang benar\n- [ ] State management terimplementasi dan digunakan secara konsisten\n- [ ] Integrasi API (fetch/axios) minimal untuk 2 fitur\n- [ ] Responsive layout — tampilan baik di viewport ≤768px\n- [ ] README.md lengkap: deskripsi project, cara install, cara menjalankan\n- [ ] Kode bersih: tidak ada console.log berlebihan, struktur folder rapi\n- [ ] Tidak ada error di console browser saat menjalankan aplikasi\n- [ ] Deploy URL aktif dan bisa diakses (opsional, nilai tambah)',
    30
),
(
    '00000001-0000-0000-0000-000000000002',
    'Capstone Project: REST API Lengkap',
    E'## Instruksi Final Project — Backend Developer\n\nBangun sebuah **REST API production-ready** menggunakan bahasa dan framework pilihanmu (Node.js/Express, Python/FastAPI, Go/Gin, Rust/Axum, dll).\n\n### Fitur Wajib\n1. **Autentikasi JWT** — register, login, refresh token\n2. **CRUD lengkap** untuk minimal 2 resource yang berelasi (contoh: users + posts, products + categories)\n3. **Validasi input** pada semua endpoint yang menerima data\n4. **Error handling** yang konsisten dengan format response standar\n5. **Database** relasional (PostgreSQL/MySQL) dengan migration\n6. **Dokumentasi API** — Swagger/OpenAPI atau koleksi Postman\n\n### Persyaratan Tambahan\n- Gunakan environment variables untuk konfigurasi sensitif\n- Implementasi pagination pada endpoint list\n- Minimal 1 endpoint dengan query filter/search\n- Rate limiting atau middleware keamanan dasar\n\n### Format Submission\nUpload file **ZIP** berisi source code, README.md, dan dokumentasi API.',
    E'## Checklist Penilaian\n\n- [ ] Autentikasi JWT berfungsi (register, login, protected routes)\n- [ ] CRUD lengkap untuk minimal 2 resource berelasi\n- [ ] Validasi input pada semua endpoint POST/PUT/PATCH\n- [ ] Error handling konsisten (format error response standar)\n- [ ] Database migration yang bisa direproduksi\n- [ ] Dokumentasi API (Swagger/Postman collection)\n- [ ] README.md: deskripsi, cara setup, cara menjalankan\n- [ ] Environment variables untuk secret/config',
    30
),
(
    '00000001-0000-0000-0000-000000000003',
    'Capstone Project: High-Fidelity Design System',
    E'## Instruksi Final Project — UI/UX Designer\n\nRancang **design system dan prototype interaktif** untuk sebuah aplikasi (mobile atau web) dari awal sampai high-fidelity.\n\n### Deliverables\n1. **User Research Summary** — persona, user journey map (minimal 1 flow utama)\n2. **Wireframe** — low-fidelity wireframe untuk semua halaman utama\n3. **High-Fidelity Mockup** — desain detail dengan warna, tipografi, dan imagery\n4. **Prototype Interaktif** — flow utama yang bisa diklik (Figma/Adobe XD)\n5. **Design System** — komponen library: button, input, card, typography scale, color palette\n\n### Persyaratan\n- Minimal 8 screen/halaman\n- Konsistensi visual antar halaman\n- Accessibility consideration (kontras warna, ukuran touch target)\n- Responsive variant (mobile + desktop) untuk minimal 3 halaman\n\n### Format Submission\nUpload file **ZIP** berisi: export PDF semua screen, aset desain, dan link Figma (di README.md).',
    E'## Checklist Penilaian\n\n- [ ] User research: persona dan user journey map\n- [ ] Wireframe low-fidelity untuk semua halaman utama\n- [ ] High-fidelity mockup minimal 8 screen\n- [ ] Prototype interaktif (link Figma yang bisa diklik)\n- [ ] Design system: komponen, color palette, typography\n- [ ] Responsive variant (mobile + desktop) untuk 3 halaman\n- [ ] Accessibility: kontras warna AA, touch target ≥44px\n- [ ] README.md dengan deskripsi project dan link prototype',
    25
),
(
    '00000001-0000-0000-0000-000000000004',
    'Capstone Project: End-to-End Data Analysis',
    E'## Instruksi Final Project — Data Analyst\n\nLakukan **analisis data end-to-end** mulai dari pengumpulan data hingga presentasi insight.\n\n### Tahapan\n1. **Data Collection** — pilih dataset publik (Kaggle, UCI, data.go.id) atau scrape sendiri\n2. **Data Cleaning** — handle missing values, outliers, tipe data\n3. **Exploratory Data Analysis (EDA)** — statistik deskriptif, distribusi, korelasi\n4. **Visualisasi** — minimal 8 chart/grafik yang informatif (matplotlib/seaborn/plotly)\n5. **Insight & Rekomendasi** — minimal 5 temuan bisnis/analitis yang actionable\n6. **Laporan** — ringkasan eksekutif dalam format markdown atau PDF\n\n### Persyaratan Teknis\n- Gunakan Python (Jupyter Notebook) atau R Markdown\n- Kode harus reproducible — bisa dijalankan dari awal\n- Dataset disertakan dalam submission (atau link download di README)\n\n### Format Submission\nUpload file **ZIP** berisi: notebook, dataset, laporan ringkas, dan README.md.',
    E'## Checklist Penilaian\n\n- [ ] Dataset terdokumentasi: sumber, deskripsi kolom\n- [ ] Data cleaning: handling missing values dan outliers\n- [ ] EDA: minimal 8 visualisasi yang informatif\n- [ ] Analisis statistik: korelasi, distribusi, trend\n- [ ] Insight: minimal 5 temuan actionable\n- [ ] Notebook berjalan dari awal sampai akhir tanpa error\n- [ ] Laporan ringkasan eksekutif\n- [ ] README.md: deskripsi project, cara menjalankan',
    25
),
(
    '00000001-0000-0000-0000-000000000005',
    'Capstone Project: Mobile App Lengkap',
    E'## Instruksi Final Project — Mobile Developer\n\nKembangkan sebuah **aplikasi mobile lengkap** menggunakan framework pilihanmu (Flutter, React Native, atau native Android/iOS).\n\n### Fitur Wajib\n1. **Minimal 5 halaman** dengan navigasi yang smooth\n2. **Autentikasi** — login/register (boleh pakai Firebase Auth atau custom API)\n3. **Konsumsi REST API** — fetch dan tampilkan data dari backend\n4. **Local Storage** — simpan data offline (SharedPreferences/Hive/SQLite)\n5. **State Management** — Provider/Bloc/GetX (Flutter) atau Redux/Context (RN)\n6. **UI yang polished** — animasi transisi, loading state, error state\n\n### Persyaratan Tambahan\n- Responsive layout untuk berbagai ukuran layar\n- Minimal 1 fitur yang menggunakan sensor/hardware (kamera, lokasi, notifikasi)\n- Handle edge case: no internet, empty state, error state\n\n### Format Submission\nUpload file **ZIP** berisi source code lengkap, README.md, dan screenshot/recording.',
    E'## Checklist Penilaian\n\n- [ ] Minimal 5 halaman dengan navigasi yang benar\n- [ ] Autentikasi (login/register) berfungsi\n- [ ] Integrasi REST API untuk minimal 2 fitur\n- [ ] Local storage/offline capability\n- [ ] State management terimplementasi\n- [ ] UI polished: loading state, error state, animasi\n- [ ] Handle edge case: no internet, empty state\n- [ ] README.md: deskripsi, cara build dan run, screenshot',
    30
);
