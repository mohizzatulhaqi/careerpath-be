-- Seed: Learning modules for UI/UX, Data Analyst, Mobile, Project Manager
-- Run: psql "$DATABASE_URL" -f seed_roles_content.sql

DO $$
DECLARE
  r_uiux UUID := '00000001-0000-0000-0000-000000000003';
  r_da   UUID := '00000001-0000-0000-0000-000000000004';
  r_mob  UUID := '00000001-0000-0000-0000-000000000005';
  r_pm   UUID := 'f1fe3220-fba7-4663-be7c-75dc13c2de79';
  m UUID; s UUID; q UUID;
BEGIN

IF EXISTS (SELECT 1 FROM learning_modules WHERE role_id = r_uiux LIMIT 1) THEN
  RAISE NOTICE 'Role content already seeded — skipping.'; RETURN;
END IF;

-- ═══════════════════════ UI/UX DESIGNER ═══════════════════════

-- Module 1: Prinsip Desain Visual
INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_uiux, 'Prinsip Desain Visual', 'Pelajari elemen dasar desain: tipografi, warna, layout, dan hierarki visual.', 1)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Tipografi & Warna', E'# Tipografi & Warna\n\nTipografi dan warna adalah fondasi identitas visual produk digital.\n\n## Tipografi\n- **Font pairing**: kombinasikan serif dan sans-serif\n- **Type scale**: gunakan skala konsisten (12, 14, 16, 20, 24, 32px)\n- **Line height**: 1.4–1.6× ukuran font untuk keterbacaan optimal\n\n## Warna\n- **Kontras WCAG AA**: minimal 4.5:1 untuk teks normal\n- **60-30-10 rule**: 60% warna primer, 30% sekunder, 10% aksen\n- **Warna semantik**: merah = error, hijau = sukses, kuning = peringatan', 1, 15)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Rasio kontras minimum WCAG AA untuk teks normal adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'2.5:1',false,1),(q,'4.5:1',true,2),(q,'3.0:1',false,3),(q,'7.0:1',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Jarak antar baris teks dalam tipografi disebut...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Kerning',false,1),(q,'Line height',true,2),(q,'Tracking',false,3),(q,'Weight',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Warna yang saling berlawanan di color wheel disebut...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Analogous',false,1),(q,'Complementary',true,2),(q,'Triadic',false,3),(q,'Monochromatic',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Layout & Grid System', E'# Layout & Grid System\n\nGrid memberikan konsistensi, keterbacaan, dan ritme visual.\n\n## 12-Column Grid\nPaling umum digunakan karena bisa dibagi 2, 3, 4, dan 6 kolom.\n\n## Spacing System\nGunakan kelipatan 8px: 8, 16, 24, 32, 48, 64px.\n\n## Visual Hierarchy\n- Ukuran: elemen besar terlihat lebih penting\n- Kontras: warna terang menarik perhatian lebih\n- Posisi: atas-kiri dibaca lebih dulu (F-pattern)', 2, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Jumlah kolom paling umum dalam grid system adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'8',false,1),(q,'12',true,2),(q,'6',false,3),(q,'16',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Whitespace dalam desain berfungsi untuk...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menghemat ruang',false,1),(q,'Meningkatkan keterbacaan dan fokus',true,2),(q,'Mengisi ruang kosong',false,3),(q,'Mengurangi elemen',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Prinsip menempatkan elemen penting di posisi paling mencolok disebut...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Proximity',false,1),(q,'Visual hierarchy',true,2),(q,'Alignment',false,3),(q,'Repetition',false,4);

-- Final quiz Module 1 UIUX
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Tujuan type scale dalam design system adalah...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membatasi jumlah font',false,1),(q,'Konsistensi ukuran teks antar komponen',true,2),(q,'Mempercepat desain',false,3),(q,'Mengurangi warna',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Elemen yang pertama kali dilihat user adalah...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Warna background',false,1),(q,'Elemen dengan ukuran dan kontras tertinggi',true,2),(q,'Teks terkecil',false,3),(q,'Elemen sudut kiri bawah',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Padding berbeda dengan margin karena...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Padding lebih besar',false,1),(q,'Padding adalah ruang di dalam elemen, margin di luar',true,2),(q,'Margin hanya untuk teks',false,3),(q,'Keduanya sama',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Gestalt principles penting dalam desain karena...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Untuk membuat animasi',false,1),(q,'Menjelaskan cara otak manusia mengelompokkan elemen visual',true,2),(q,'Untuk memilih warna',false,3),(q,'Mengoptimalkan loading time',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Accessibility dalam UI terutama bertujuan untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mempercantik tampilan',false,1),(q,'Memastikan produk bisa digunakan semua orang termasuk difabel',true,2),(q,'Meningkatkan performa',false,3),(q,'Mengurangi elemen',false,4);

-- Module 2: UX Research & Prototyping
INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_uiux, 'UX Research & Prototyping', 'Kuasai metode riset pengguna dan pembuatan prototype interaktif dari wireframe hingga high-fidelity.', 2)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'User Research & Persona', E'# User Research & Persona\n\n## Metode Riset\n- **User Interview** — wawancara mendalam, ideal untuk memahami motivasi\n- **Survey** — data kuantitatif dari banyak responden\n- **Usability Testing** — amati pengguna langsung menggunakan produk\n- **Card Sorting** — pahami mental model pengguna\n\n## User Persona\nKarakter fiktif yang mewakili segmen pengguna nyata. Berisi: nama, usia, pekerjaan, goals, frustrations, dan behavior.\n\n## User Journey Map\nVisualisasi pengalaman pengguna dari awal hingga akhir interaksi, mencakup touchpoints, emosi, dan pain points.', 1, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'User persona adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Akun user di aplikasi',false,1),(q,'Representasi fiktif dari segmen pengguna nyata',true,2),(q,'Database pengguna',false,3),(q,'Login system',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Metode paling efektif untuk memahami mental model pengguna adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'A/B Testing',false,1),(q,'User Interview mendalam',true,2),(q,'Web analytics',false,3),(q,'Survey skala besar',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'User Journey Map digunakan untuk...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Memetakan alur kode aplikasi',false,1),(q,'Memvisualisasikan pengalaman pengguna dari awal hingga akhir interaksi',true,2),(q,'Membuat sitemap website',false,3),(q,'Mendesain database',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Wireframe & Prototype di Figma', E'# Wireframe & Prototype di Figma\n\n## Wireframe (Low-fidelity)\nSketsa kasar yang menentukan tata letak tanpa visual detail. Tools: kertas, Figma, Balsamiq.\n\n## Mockup (High-fidelity)\nDesain detail dengan warna, font, dan imagery nyata.\n\n## Prototype Interaktif\nTambahkan koneksi antar frame di Figma agar desain bisa diklik dan dipresentasikan ke stakeholder sebelum development.\n\n## Usability Testing\nUji prototype ke 5–7 pengguna untuk menemukan masalah UX sebelum development dimulai.', 2, 25)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Perbedaan wireframe dan mockup adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Wireframe lebih berwarna',false,1),(q,'Wireframe low-fidelity tanpa detail, mockup high-fidelity dengan warna lengkap',true,2),(q,'Mockup tidak bisa diklik',false,3),(q,'Keduanya sama',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Fitur Figma yang membuat desain menjadi interaktif adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Auto Layout',false,1),(q,'Prototype connections & interactions',true,2),(q,'Components',false,3),(q,'Variants',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Tujuan usability testing pada prototype adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membuat prototype lebih indah',false,1),(q,'Menemukan masalah UX sebelum development dimulai',true,2),(q,'Memilih warna yang tepat',false,3),(q,'Mempercepat proses desain',false,4);

-- Final quiz Module 2 UIUX
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Design thinking terdiri dari berapa tahap?', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'3',false,1),(q,'5',true,2),(q,'4',false,3),(q,'7',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, '"Empathize" dalam design thinking berarti...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membuat desain yang emosional',false,1),(q,'Memahami kebutuhan dan sudut pandang pengguna secara mendalam',true,2),(q,'Mendesain dengan empati antar tim',false,3),(q,'Menulis kode dengan empati',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Information Architecture berkaitan dengan...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Arsitektur gedung kantor',false,1),(q,'Pengorganisasian konten agar mudah ditemukan pengguna',true,2),(q,'Database design',false,3),(q,'Struktur kode aplikasi',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Card sorting digunakan untuk...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengurutkan kartu nama tim',false,1),(q,'Memahami mental model pengguna dalam mengelompokkan informasi',true,2),(q,'Menyortir komponen Figma',false,3),(q,'Membuat kartu nama digital',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Atomic design membagi komponen menjadi...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Besar dan kecil',false,1),(q,'Atom, molecule, organism, template, page',true,2),(q,'Component dan variant',false,3),(q,'Mobile dan desktop',false,4);

-- ═══════════════════════ DATA ANALYST ═══════════════════════

INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_da, 'SQL untuk Analisis Data', 'Kuasai SQL dari query dasar hingga aggregasi, join, dan subquery untuk mengekstrak insight dari database.', 1)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Query Dasar & Filter', E'# Query Dasar SQL\n\n## SELECT, FROM, WHERE\n```sql\nSELECT nama, email FROM users WHERE is_active = true;\nSELECT * FROM orders WHERE total > 500000 AND status = ''completed'';\n```\n\n## ORDER BY & LIMIT\n```sql\nSELECT nama, gaji FROM karyawan ORDER BY gaji DESC LIMIT 10;\n```\n\n## LIKE & IN\n```sql\nSELECT * FROM users WHERE email LIKE ''%@gmail.com'';\nSELECT * FROM products WHERE kategori IN (''elektronik'', ''fashion'');\n```', 1, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Perintah SQL untuk mengambil semua kolom dari tabel adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'GET * FROM users',false,1),(q,'SELECT * FROM users',true,2),(q,'FETCH users',false,3),(q,'READ users',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Klausa untuk memfilter baris berdasarkan kondisi adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'FILTER',false,1),(q,'WHERE',true,2),(q,'HAVING',false,3),(q,'CONDITION',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Untuk mengurutkan hasil dari terbesar ke terkecil digunakan...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'ORDER BY kolom ASC',false,1),(q,'ORDER BY kolom DESC',true,2),(q,'SORT BY kolom',false,3),(q,'GROUP BY kolom DESC',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Aggregasi & JOIN', E'# Aggregasi & JOIN\n\n## Fungsi Agregat\n```sql\nSELECT COUNT(*) AS total, SUM(total) AS pendapatan,\n       AVG(total) AS rata_rata, MAX(total) AS tertinggi\nFROM orders WHERE status = ''completed'';\n```\n\n## GROUP BY & HAVING\n```sql\nSELECT kategori, COUNT(*) as jumlah\nFROM produk GROUP BY kategori HAVING COUNT(*) > 5;\n```\n\n## JOIN\n```sql\n-- INNER JOIN\nSELECT o.id, u.nama, o.total\nFROM orders o INNER JOIN users u ON u.id = o.user_id;\n\n-- LEFT JOIN\nSELECT u.nama, COUNT(o.id) as total_order\nFROM users u LEFT JOIN orders o ON o.user_id = u.id\nGROUP BY u.id, u.nama;\n```', 2, 25)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Fungsi agregat untuk menghitung jumlah baris adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'SUM()',false,1),(q,'COUNT()',true,2),(q,'TOTAL()',false,3),(q,'NUMBER()',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'HAVING berbeda dengan WHERE karena...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'HAVING lebih cepat',false,1),(q,'HAVING memfilter setelah GROUP BY, WHERE sebelum agregasi',true,2),(q,'HAVING hanya untuk JOIN',false,3),(q,'WHERE tidak bisa pakai GROUP BY',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'LEFT JOIN mengembalikan...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Hanya baris yang cocok di kedua tabel',false,1),(q,'Semua baris tabel kiri meski tidak ada pasangan di tabel kanan',true,2),(q,'Semua baris dari kedua tabel',false,3),(q,'Hanya baris dari tabel kanan',false,4);

-- Final quiz Module 1 DA
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'CTE (Common Table Expression) digunakan untuk...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membuat tabel permanen',false,1),(q,'Mendefinisikan subquery sementara yang bisa dipakai ulang dalam query yang sama',true,2),(q,'Mengganti INDEX',false,3),(q,'Membuat stored procedure',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'EXPLAIN ANALYZE di PostgreSQL digunakan untuk...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengekspor data ke CSV',false,1),(q,'Menganalisis query plan dan menemukan bottleneck performa',true,2),(q,'Backup database',false,3),(q,'Membuat dokumentasi',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'NULL dalam SQL berarti...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Nilai nol (0)',false,1),(q,'Tidak ada nilai / tidak diketahui — bukan nilai kosong',true,2),(q,'String kosong',false,3),(q,'False',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Window function RANK() berbeda dari ROW_NUMBER() karena...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'RANK() lebih cepat',false,1),(q,'RANK() memberi nilai sama untuk baris dengan nilai identik (tied ranks)',true,2),(q,'ROW_NUMBER() lebih akurat',false,3),(q,'Keduanya identik',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'INDEX database berfungsi untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menyimpan data cadangan',false,1),(q,'Mempercepat pencarian pada kolom yang sering di-filter',true,2),(q,'Mengamankan data',false,3),(q,'Mengkompres tabel',false,4);

-- Module 2: Visualisasi & Storytelling Data
INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_da, 'Visualisasi & Storytelling Data', 'Pelajari cara memilih chart yang tepat, membangun dashboard, dan menyampaikan insight secara efektif.', 2)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Memilih Chart yang Tepat', E'# Memilih Chart yang Tepat\n\n| Tujuan | Chart |\n|--------|-------|\n| Perbandingan | Bar / Column chart |\n| Tren waktu | Line / Area chart |\n| Proporsi | Pie / Donut / Stacked bar |\n| Distribusi | Histogram / Box plot |\n| Korelasi | Scatter plot |\n\n## Anti-Pattern\n- Pie chart > 5 segmen → susah dibaca\n- 3D chart → mendistorsi persepsi\n- Dual-axis → membingungkan audiens', 1, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Chart terbaik untuk tren penjualan 12 bulan adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Pie chart',false,1),(q,'Line chart',true,2),(q,'Scatter plot',false,3),(q,'Histogram',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Untuk membandingkan nilai antar kategori, chart paling efektif adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Line chart',false,1),(q,'Bar chart',true,2),(q,'Pie chart',false,3),(q,'Area chart',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Scatter plot digunakan untuk menunjukkan...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Proporsi dari total',false,1),(q,'Korelasi antara dua variabel numerik',true,2),(q,'Tren waktu',false,3),(q,'Distribusi frekuensi',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Dashboard & Data Storytelling', E'# Dashboard & Data Storytelling\n\n## Prinsip Dashboard Efektif\n1. **Satu tujuan, satu audiens**\n2. **KPI di atas** — angka paling penting pertama\n3. **Context beats data** — bandingkan dengan target atau periode lalu\n4. **Interaktivitas** — filter, drill-down, date range\n\n## Data Storytelling\nUrutan narasi: **Situasi → Komplikasi → Resolusi**\n- Mulai dari pertanyaan bisnis, bukan dari data\n- Highlight anomali dan outlier\n- Kesimpulan harus actionable', 2, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'KPI adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Jenis chart khusus',false,1),(q,'Metrik kunci yang mengukur keberhasilan terhadap tujuan bisnis',true,2),(q,'Alat visualisasi data',false,3),(q,'Format file data',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, '"Komplikasi" dalam data storytelling merujuk pada...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Error dalam data',false,1),(q,'Masalah atau tantangan yang ditemukan dari analisis',true,2),(q,'Bug di dashboard',false,3),(q,'Data yang hilang',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Terlalu banyak detail di dashboard menyebabkan...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Data lebih akurat',false,1),(q,'Information overload — audiens sulit menemukan insight utama',true,2),(q,'Dashboard lebih cepat',false,3),(q,'Warna lebih bervariasi',false,4);

-- Final quiz Module 2 DA
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Outlier dalam data sebaiknya...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Selalu dihapus',false,1),(q,'Diinvestigasi dulu: bisa error atau insight berharga',true,2),(q,'Diabaikan',false,3),(q,'Diganti rata-rata',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Median lebih baik dari mean ketika...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Data sangat banyak',false,1),(q,'Ada outlier yang signifikan — median lebih tahan terhadap nilai ekstrem',true,2),(q,'Data dalam persen',false,3),(q,'Menggunakan kategori',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Cohort analysis berguna untuk...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menganalisis kode program',false,1),(q,'Melacak perilaku grup pengguna yang bergabung di periode yang sama',true,2),(q,'Mengklasifikasikan produk',false,3),(q,'Laporan keuangan',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'A/B Testing digunakan untuk...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Memilih warna dashboard',false,1),(q,'Mengukur dampak perubahan secara statistik dengan membandingkan dua varian',true,2),(q,'Backup database',false,3),(q,'Membersihkan data duplikat',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Data pipeline berfungsi untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mempercantik visualisasi',false,1),(q,'Mengotomasi alur dari pengumpulan, transformasi, hingga penyimpanan data',true,2),(q,'Mengamankan database',false,3),(q,'Membuat laporan PDF',false,4);

-- ═══════════════════════ MOBILE DEVELOPER ═══════════════════════

INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_mob, 'Dasar Flutter & Dart', 'Pelajari fondasi pengembangan mobile cross-platform: widget, layout, dan dasar bahasa Dart.', 1)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Dart & Widget Dasar Flutter', E'# Dart & Widget Dasar Flutter\n\n## Tipe Data Dart\n```dart\nString nama = "Flutter";\nint umur = 5;\ndouble versi = 3.10;\nbool isStable = true;\nList<String> list = ["a", "b"];\n```\n\n## Widget Dasar\n```dart\nScaffold(\n  appBar: AppBar(title: Text("Hello")),\n  body: Center(\n    child: Column(children: [\n      Text("Halo!", style: TextStyle(fontSize: 24)),\n      ElevatedButton(onPressed: () {}, child: Text("Klik")),\n    ]),\n  ),\n)\n```\n\n## StatelessWidget vs StatefulWidget\n- **Stateless**: UI tidak berubah setelah dibuat\n- **Stateful**: UI bisa berubah saat state di-update via setState()', 1, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Bahasa pemrograman Flutter adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Kotlin',false,1),(q,'Dart',true,2),(q,'JavaScript',false,3),(q,'Swift',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Widget yang menyediakan struktur halaman dasar dengan AppBar adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Container',false,1),(q,'Scaffold',true,2),(q,'Column',false,3),(q,'Stack',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Perbedaan StatelessWidget dan StatefulWidget adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Stateless lebih cepat render',false,1),(q,'StatefulWidget bisa berubah state-nya saat runtime, Stateless tidak',true,2),(q,'Stateless lebih baru',false,3),(q,'Keduanya sama',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Layout & Navigasi Antar Halaman', E'# Layout & Navigasi di Flutter\n\n## Widget Layout\n- `Column` — vertikal\n- `Row` — horizontal\n- `Stack` — tumpuk (z-axis)\n- `Expanded` — isi sisa ruang\n- `ListView` — scroll list\n\n## Navigasi\n```dart\n// Pindah ke halaman baru\nNavigator.push(context,\n  MaterialPageRoute(builder: (_) => DetailPage()));\n\n// Kembali\nNavigator.pop(context);\n\n// Kirim data ke halaman baru\nNavigator.push(context,\n  MaterialPageRoute(builder: (_) => DetailPage(id: item.id)));\n```', 2, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Widget untuk menyusun children secara vertikal adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Row',false,1),(q,'Column',true,2),(q,'Stack',false,3),(q,'Wrap',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Untuk pindah ke halaman baru di Flutter menggunakan...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Router.go()',false,1),(q,'Navigator.push()',true,2),(q,'Page.open()',false,3),(q,'Screen.navigate()',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Widget Expanded berfungsi untuk...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membesar ukuran font',false,1),(q,'Mengisi sisa ruang yang tersedia dalam Row atau Column',true,2),(q,'Menampilkan konten yang bisa di-scroll',false,3),(q,'Membuat animasi expand',false,4);

-- Final quiz Module 1 Mobile
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Hot reload berguna untuk...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Restart ulang HP',false,1),(q,'Melihat perubahan UI secara instan tanpa restart aplikasi penuh',true,2),(q,'Membersihkan cache',false,3),(q,'Update Flutter SDK',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'setState() digunakan untuk...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menyimpan data ke database',false,1),(q,'Memberitahu Flutter bahwa state berubah dan UI perlu di-rebuild',true,2),(q,'Mengatur warna aplikasi',false,3),(q,'Membuat animasi',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'pubspec.yaml berfungsi untuk...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menyimpan data pengguna',false,1),(q,'Mendefinisikan dependensi, aset, dan metadata proyek',true,2),(q,'Konfigurasi server',false,3),(q,'Menyimpan kode Dart',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'FutureBuilder digunakan ketika...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membuat tombol animasi',false,1),(q,'Menampilkan UI yang bergantung pada operasi async seperti fetch API',true,2),(q,'Membangun layout grid',false,3),(q,'Membuat drawer menu',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'MediaQuery.of(context).size digunakan untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengambil ukuran widget tertentu',false,1),(q,'Mendapatkan dimensi layar perangkat untuk layout responsif',true,2),(q,'Mengatur ukuran font',false,3),(q,'Mengecek orientasi saja',false,4);

-- Module 2: State Management & API
INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_mob, 'State Management & Integrasi API', 'Kuasai Provider untuk state management dan integrasi REST API menggunakan http package.', 2)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'State Management dengan Provider', E'# State Management dengan Provider\n\n```dart\n// Model\nclass CartModel extends ChangeNotifier {\n  List<Item> _items = [];\n  List<Item> get items => _items;\n\n  void add(Item item) {\n    _items.add(item);\n    notifyListeners();\n  }\n}\n\n// Setup di main.dart\nChangeNotifierProvider(create: (_) => CartModel(), child: MyApp())\n\n// Di widget — watch: rebuild otomatis, read: sekali pakai\nfinal cart = context.watch<CartModel>();\nfinal cart = context.read<CartModel>();\n```', 1, 25)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'notifyListeners() berfungsi untuk...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengirim notifikasi push',false,1),(q,'Memberitahu widget listener bahwa state telah berubah',true,2),(q,'Mencetak log ke console',false,3),(q,'Menyimpan data ke SharedPreferences',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Perbedaan context.watch() dan context.read() adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'watch() lebih cepat',false,1),(q,'watch() rebuild widget saat state berubah, read() hanya membaca sekali',true,2),(q,'read() hanya untuk ChangeNotifier',false,3),(q,'Keduanya identik',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'ChangeNotifier digunakan di Provider karena...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Wajib dipakai di Flutter',false,1),(q,'Menyediakan mekanisme notifikasi ke listener saat data berubah',true,2),(q,'Lebih ringan dari setState',false,3),(q,'Gratis tanpa dependensi',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Integrasi REST API di Flutter', E'# Integrasi REST API\n\n```dart\nimport ''package:http/http.dart'' as http;\nimport ''dart:convert'';\n\nFuture<List<Post>> fetchPosts() async {\n  final res = await http.get(\n    Uri.parse(''https://api.example.com/posts''),\n    headers: {''Authorization'': ''Bearer $token''},\n  );\n  if (res.statusCode == 200) {\n    return (jsonDecode(res.body) as List)\n        .map((j) => Post.fromJson(j)).toList();\n  }\n  throw Exception(''Gagal memuat data: ${res.statusCode}'');\n}\n```\n\n## Model\n```dart\nclass Post {\n  final int id; final String title;\n  Post({required this.id, required this.title});\n  factory Post.fromJson(Map<String, dynamic> j) =>\n      Post(id: j[''id''], title: j[''title'']);\n}\n```', 2, 25)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'jsonDecode() digunakan untuk...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengenkripsi data',false,1),(q,'Mengubah string JSON menjadi objek Dart (Map/List)',true,2),(q,'Mengompresi response',false,3),(q,'Memvalidasi JSON',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'HTTP status code 401 berarti...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Data tidak ditemukan',false,1),(q,'Unauthorized — token tidak valid atau belum login',true,2),(q,'Server error',false,3),(q,'Request berhasil',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'factory constructor di Dart biasanya digunakan untuk...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membuat animasi',false,1),(q,'Membuat instance dari Map JSON — pola fromJson',true,2),(q,'Menghancurkan objek',false,3),(q,'Mengakses static member',false,4);

-- Final quiz Module 2 Mobile
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'SharedPreferences digunakan untuk...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Berbagi data antar aplikasi',false,1),(q,'Menyimpan data sederhana (key-value) secara persisten di perangkat',true,2),(q,'Mengelola state global',false,3),(q,'Enkripsi data sensitif',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'async/await penting dalam integrasi API karena...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membuat request lebih cepat',false,1),(q,'Mencegah UI freeze saat menunggu response dari jaringan',true,2),(q,'Menghemat baterai',false,3),(q,'Mengurangi ukuran APK',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'GoRouter lebih disarankan dari Navigator 1.0 karena...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'GoRouter lebih ringan',false,1),(q,'Mendukung deep linking dan URL-based routing yang lebih maintainable',true,2),(q,'Navigator 1.0 deprecated',false,3),(q,'GoRouter gratis',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Flutter flavor berguna untuk...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menambahkan rasa pada UI',false,1),(q,'Membuat varian build (dev/staging/prod) dari satu codebase',true,2),(q,'Optimasi APK size',false,3),(q,'Menjalankan test otomatis',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Interceptor di Dio berguna untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Memblokir request tertentu',false,1),(q,'Menambahkan auth header, logging, atau retry otomatis ke semua request',true,2),(q,'Mengompres response',false,3),(q,'Mengubah base URL',false,4);

-- ═══════════════════════ PROJECT MANAGER ═══════════════════════

INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_pm, 'Agile & Scrum untuk PM', 'Pahami framework Agile dan Scrum: roles, artifacts, ceremonies, dan cara mengelola sprint secara efektif.', 1)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Agile Mindset & Scrum Framework', E'# Agile & Scrum\n\n## 4 Nilai Agile Manifesto\n1. **Individu dan interaksi** > proses dan tools\n2. **Software yang berjalan** > dokumentasi komprehensif\n3. **Kolaborasi pelanggan** > negosiasi kontrak\n4. **Merespons perubahan** > mengikuti rencana\n\n## Scrum Roles\n- **Product Owner** — kelola backlog, prioritas fitur\n- **Scrum Master** — fasilitator, hilangkan hambatan\n- **Dev Team** — 3–9 orang, cross-functional\n\n## Artifacts\n- **Product Backlog** — semua kebutuhan produk\n- **Sprint Backlog** — item yang dikerjakan sprint ini\n- **Increment** — hasil yang siap rilis tiap sprint', 1, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Yang bertanggung jawab mengelola Product Backlog adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Scrum Master',false,1),(q,'Product Owner',true,2),(q,'Development Team',false,3),(q,'Stakeholder',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Sprint dalam Scrum adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Tools manajemen proyek',false,1),(q,'Iterasi pengembangan dengan durasi tetap (1–4 minggu) yang menghasilkan increment',true,2),(q,'Meeting mingguan tim',false,3),(q,'Dokumen perencanaan',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Fungsi utama Scrum Master adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menulis kode fitur',false,1),(q,'Memfasilitasi proses Scrum dan menghilangkan hambatan tim',true,2),(q,'Menentukan prioritas fitur',false,3),(q,'Mengelola anggaran',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Sprint Ceremonies & User Story', E'# Sprint Ceremonies & User Story\n\n## 4 Scrum Ceremonies\n| Ceremony | Tujuan | Durasi |\n|----------|--------|--------|\n| Sprint Planning | Pilih backlog item | 8 jam/sprint |\n| Daily Scrum | Sinkronisasi harian | 15 menit |\n| Sprint Review | Demo ke stakeholder | 4 jam/sprint |\n| Sprint Retrospective | Evaluasi proses | 3 jam/sprint |\n\n## User Story\n```\nSebagai [persona],\nSaya ingin [aksi],\nAgar [manfaat].\n```\n\n## Definition of Done\n- Code review selesai\n- Unit test lulus\n- QA testing selesai\n- Deployed ke staging', 2, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Tujuan Sprint Retrospective adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Demo produk ke klien',false,1),(q,'Evaluasi proses tim dan mencari perbaikan untuk sprint berikutnya',true,2),(q,'Merencanakan fitur baru',false,3),(q,'Review kode program',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Format user story yang benar adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Detail implementasi teknis',false,1),(q,'"Sebagai [persona], Saya ingin [aksi], Agar [manfaat]"',true,2),(q,'Daftar bug yang perlu diperbaiki',false,3),(q,'Spesifikasi API endpoint',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Definition of Done berguna untuk...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menentukan gaji tim',false,1),(q,'Memastikan seluruh tim punya pemahaman sama tentang kriteria selesai',true,2),(q,'Mengukur kecepatan coding',false,3),(q,'Mendefinisikan jam kerja',false,4);

-- Final quiz Module 1 PM
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Velocity dalam Scrum digunakan untuk...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengukur kecepatan internet',false,1),(q,'Mengukur jumlah story point yang selesai per sprint untuk merencanakan sprint berikutnya',true,2),(q,'Mengukur kecepatan deployment',false,3),(q,'Menilai performa individu',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Story point mewakili...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Jam kerja yang dibutuhkan',false,1),(q,'Ukuran relatif kompleksitas dan effort yang dibutuhkan untuk menyelesaikan story',true,2),(q,'Jumlah baris kode',false,3),(q,'Biaya pengembangan',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Kanban berbeda dari Scrum karena...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Kanban menggunakan sprint',false,1),(q,'Kanban tidak punya iterasi tetap — work items mengalir kontinu dengan WIP limit',true,2),(q,'Scrum tidak punya backlog',false,3),(q,'Kanban hanya untuk tim kecil',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Impediment dalam Scrum adalah...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Fitur paling prioritas',false,1),(q,'Hambatan yang menghalangi tim menyelesaikan pekerjaan',true,2),(q,'Anggota tim baru',false,3),(q,'Sprint yang gagal',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Burn-down chart digunakan untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Memvisualisasikan biaya proyek',false,1),(q,'Melacak sisa pekerjaan vs waktu yang tersisa dalam sprint',true,2),(q,'Menampilkan kecepatan server',false,3),(q,'Mengukur kepuasan pengguna',false,4);

-- Module 2: Komunikasi & Manajemen Risiko
INSERT INTO learning_modules (role_id, title, description, order_index)
VALUES (r_pm, 'Komunikasi & Manajemen Risiko', 'Kelola stakeholder, buat dokumentasi proyek yang efektif, dan identifikasi serta mitigasi risiko.', 2)
RETURNING id INTO m;

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Stakeholder Management & Komunikasi', E'# Stakeholder Management\n\n## Power-Interest Matrix\n| | Low Interest | High Interest |\n|-|-------------|---------------|\n| **High Power** | Keep Satisfied | Manage Closely |\n| **Low Power** | Monitor | Keep Informed |\n\n## Tips Komunikasi Efektif\n1. Sesuaikan bahasa dengan audiens (teknis vs non-teknis)\n2. Over-communicate lebih baik dari under-communicate\n3. Gunakan visual — chart dan diagram lebih mudah dipahami\n4. Status report mingguan: progress, risiko, blockers\n5. Dokumentasikan keputusan — follow-up meeting dengan action items tertulis\n\n## RACI Matrix\n- **R**esponsible — yang mengerjakan\n- **A**ccountable — yang bertanggung jawab\n- **C**onsulted — yang dimintai pendapat\n- **I**nformed — yang perlu diinformasikan', 1, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Stakeholder "High Power, High Interest" diperlakukan dengan cara...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Monitor saja',false,1),(q,'Manage closely — libatkan aktif dan komunikasikan secara rutin',true,2),(q,'Keep satisfied dengan update minimal',false,3),(q,'Diabaikan',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Status report proyek yang baik mencakup...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Detail teknis seluruh kode',false,1),(q,'Progress, milestone, risiko aktif, dan blockers yang perlu keputusan',true,2),(q,'Hanya pencapaian positif',false,3),(q,'Daftar meeting yang sudah dilakukan',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'RACI matrix mendefinisikan...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Anggaran proyek',false,1),(q,'Siapa yang Responsible, Accountable, Consulted, dan Informed untuk setiap tugas',true,2),(q,'Jadwal sprint',false,3),(q,'Tools yang digunakan tim',false,4);

INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Identifikasi & Mitigasi Risiko', E'# Manajemen Risiko\n\n## Risk Register\n| Risiko | Prob. | Dampak | Skor | Mitigasi |\n|--------|-------|--------|------|----------|\n| Key dev resign | Rendah | Tinggi | Medium | Knowledge transfer rutin |\n| Scope creep | Tinggi | Tinggi | Kritis | Change control process |\n| Delay vendor | Medium | Medium | Medium | Vendor alternatif |\n\n## Strategi Mitigasi\n1. **Avoid** — ubah rencana agar risiko tidak terjadi\n2. **Transfer** — pindahkan ke pihak lain (asuransi, kontrak)\n3. **Mitigate** — kurangi probabilitas atau dampak\n4. **Accept** — terima jika dampak kecil\n\n## Scope Creep\nPenambahan fitur di luar scope tanpa change control — salah satu penyebab utama proyek gagal.', 2, 20)
RETURNING id INTO s;

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Risk score dihitung dari...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Biaya × durasi',false,1),(q,'Probabilitas × Dampak',true,2),(q,'Jumlah risiko yang ditemukan',false,3),(q,'Waktu mitigasi',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Scope creep adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Teknik memperluas scope secara terencana',false,1),(q,'Penambahan fitur di luar scope awal tanpa persetujuan formal',true,2),(q,'Pengurangan fitur karena waktu',false,3),(q,'Metode estimasi scope',false,4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (s, 'Strategi "Transfer" contohnya adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Membatalkan proyek',false,1),(q,'Asuransi aset proyek atau penalti keterlambatan dalam kontrak vendor',true,2),(q,'Mengurangi fitur',false,3),(q,'Menambah anggota tim',false,4);

-- Final quiz Module 2 PM
INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Perbedaan issue dan risk adalah...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Issue lebih besar dari risk',false,1),(q,'Risk adalah masalah potensial (belum terjadi), issue sudah terjadi',true,2),(q,'Keduanya sama',false,3),(q,'Risk hanya untuk proyek besar',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Triple constraint terdiri dari...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Scope, budget, tim',false,1),(q,'Scope, time, cost — mengubah satu memengaruhi yang lain',true,2),(q,'Quality, speed, features',false,3),(q,'Agile, Waterfall, Hybrid',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Lessons Learned dicatat untuk...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Menilai performa individu',false,1),(q,'Mendokumentasikan apa yang berhasil dan gagal agar proyek berikutnya lebih baik',true,2),(q,'Laporan ke klien',false,3),(q,'Menghitung biaya',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Milestone dalam timeline adalah...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Tugas harian tim',false,1),(q,'Titik pencapaian signifikan tanpa durasi (tanggal target kunci)',true,2),(q,'Meeting bulanan',false,3),(q,'Fitur paling kompleks',false,4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (m, 'Change control process berfungsi untuk...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES (q,'Mengubah anggota tim',false,1),(q,'Memastikan setiap perubahan scope dievaluasi dan disetujui sebelum diimplementasi',true,2),(q,'Mengupdate tools',false,3),(q,'Mengubah metodologi proyek',false,4);

-- ═══════════════════════ CAPSTONE PROJECT: PROJECT MANAGER ═══════════════════════

INSERT INTO projects (role_id, title, description, requirements, estimated_hours)
VALUES (
  r_pm,
  'Capstone Project: Rencana Proyek Digital Lengkap',
  E'## Instruksi Final Project — Project Manager\n\nBuat **dokumen rencana proyek lengkap** untuk produk digital fiktif (app mobile, web app, atau SaaS) dari inisiasi hingga rilis pertama.\n\n### Deliverables Wajib\n1. **Project Charter** — tujuan SMART, scope, stakeholder, success criteria\n2. **Product Backlog** — minimal 20 user story dengan estimasi story point (Fibonacci)\n3. **Sprint Plan** — 3 sprint (2 minggu/sprint) dengan sprint goal dan sprint backlog\n4. **Risk Register** — minimal 8 risiko dengan probabilitas, dampak, skor, dan mitigasi\n5. **Stakeholder Map** — Power-Interest Matrix dan rencana komunikasi\n6. **Project Timeline** — Gantt chart dengan milestone utama\n7. **Definition of Done** — kriteria penyelesaian yang jelas\n\n### Format Submission\nUpload **ZIP** berisi semua dokumen (PDF/Excel/Figma). Sertakan README.md yang menjelaskan struktur file.',
  E'## Checklist Penilaian\n\n- [ ] Project Charter dengan tujuan SMART dan success criteria\n- [ ] Minimal 20 user story dengan format benar dan story point\n- [ ] Sprint plan 3 sprint dengan sprint goal yang jelas\n- [ ] Risk register minimal 8 risiko dengan skor dan mitigasi\n- [ ] Stakeholder map dengan Power-Interest Matrix\n- [ ] Timeline/Gantt chart dengan minimal 5 milestone\n- [ ] Definition of Done yang konkret dan terukur\n- [ ] README.md yang menjelaskan isi dokumen',
  20
) ON CONFLICT DO NOTHING;

RAISE NOTICE '✓ Seed selesai: UIUX 2 modul, Data Analyst 2 modul, Mobile 2 modul, Project Manager 2 modul + project.';

END $$;
