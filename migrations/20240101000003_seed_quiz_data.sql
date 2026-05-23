DO $$
DECLARE
  r_fe   UUID := '00000001-0000-0000-0000-000000000001';
  r_be   UUID := '00000001-0000-0000-0000-000000000002';
  r_uiux UUID := '00000001-0000-0000-0000-000000000003';
  r_da   UUID := '00000001-0000-0000-0000-000000000004';
  r_mob  UUID := '00000001-0000-0000-0000-000000000005';

  q UUID; o1 UUID; o2 UUID; o3 UUID; o4 UUID;
BEGIN

-- ── Roles ────────────────────────────────────────────────────────────────────
INSERT INTO roles (id, code, name, description) VALUES
  (r_fe,   'frontend',     'Frontend Developer', 'Membangun antarmuka web yang interaktif dan responsif'),
  (r_be,   'backend',      'Backend Developer',  'Membangun logika server, API, dan infrastruktur aplikasi'),
  (r_uiux, 'uiux',         'UI/UX Designer',     'Mendesain pengalaman dan antarmuka pengguna yang intuitif'),
  (r_da,   'data_analyst', 'Data Analyst',        'Menganalisis data untuk menghasilkan insight bisnis yang bernilai'),
  (r_mob,  'mobile',       'Mobile Developer',   'Membangun aplikasi native atau cross-platform untuk Android dan iOS');

-- ── Q1 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Tugas apa yang paling kamu nikmati saat mengerjakan proyek?', 1)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Mendesain tampilan yang menarik dan user-friendly', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Menganalisis pola tersembunyi dalam dataset besar', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Membangun API yang efisien dan logika sisi server', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Membuat aplikasi yang berjalan mulus di smartphone', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 4),(o1, r_be, 0),(o1, r_uiux, 5),(o1, r_da, 1),(o1, r_mob, 3);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 2),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 3),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 2),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q2 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Tools atau bahasa pemrograman mana yang paling menarik bagimu?', 2)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'HTML, CSS, JavaScript / TypeScript', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Python, R, SQL, dan Tableau', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Kotlin, Swift, Flutter, atau React Native', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Node.js, Go, Java, atau PostgreSQL', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 5),(o1, r_be, 1),(o1, r_uiux, 3),(o1, r_da, 0),(o1, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 2),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 2),(o3, r_be, 1),(o3, r_uiux, 1),(o3, r_da, 0),(o3, r_mob, 5);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 1),(o4, r_be, 5),(o4, r_uiux, 0),(o4, r_da, 2),(o4, r_mob, 2);

-- ── Q3 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Output kerja yang paling membuatmu bangga adalah...?', 3)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Website yang indah, responsif, dan mudah digunakan', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Dashboard atau laporan analitik yang akurat dan insightful', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'API yang cepat, aman, dan terdokumentasi dengan baik', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Aplikasi mobile yang mulus dan konsisten di berbagai device', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 5),(o1, r_be, 0),(o1, r_uiux, 4),(o1, r_da, 1),(o1, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 1),(o2, r_be, 1),(o2, r_uiux, 3),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 2),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 3),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q4 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Kamu lebih suka menghabiskan waktu dengan...?', 4)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Riset pengguna dan membuat wireframe atau mockup', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Eksplorasi data dan menemukan insight tersembunyi', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Menulis kode server dan merancang skema database', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Optimasi animasi dan performa aplikasi di berbagai device', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 2),(o1, r_be, 0),(o1, r_uiux, 5),(o1, r_da, 2),(o1, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 1),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 3),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 3),(o4, r_be, 1),(o4, r_uiux, 2),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q5 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Ketika ada bug, kamu cenderung...?', 5)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Fokus memperbaiki tampilan yang tidak sesuai desain', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Investigasi anomali di data atau pipeline log', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Debug di sisi server, cek query plan dan response time', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Cek crash report di emulator dan baca stack trace device', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 4),(o1, r_be, 0),(o1, r_uiux, 5),(o1, r_da, 1),(o1, r_mob, 3);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 2),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 1);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 3),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 1),(o4, r_da, 1),(o4, r_mob, 5);

-- ── Q6 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Topik mana yang paling kamu minati untuk dipelajari?', 6)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'User Experience, Aksesibilitas, dan Design System', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Machine Learning, Statistik, dan Business Intelligence', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Microservices, Cloud Architecture, dan System Design', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Cross-platform dev, OS APIs, dan Mobile Performance', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 3),(o1, r_be, 0),(o1, r_uiux, 5),(o1, r_da, 1),(o1, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 2),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 2),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 1),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q7 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Teman-teman biasanya minta bantuanmu untuk...?', 7)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Bikin tampilan web yang responsif dan interaktif', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Mengolah atau memvisualisasikan data dengan rapi', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Setup server, database, atau konfigurasi deployment', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Bikin, test, atau publish aplikasi di smartphone', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 5),(o1, r_be, 1),(o1, r_uiux, 3),(o1, r_da, 0),(o1, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 1),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 2),(o3, r_mob, 1);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 2),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q8 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Proyek impianmu adalah...?', 8)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Redesign UI aplikasi populer yang antarmukanya membingungkan', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Membangun model prediksi tren pasar dari data historis', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Bangun backend yang bisa melayani jutaan concurrent user', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Publish aplikasi buatanmu ke Google Play dan App Store', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 3),(o1, r_be, 0),(o1, r_uiux, 5),(o1, r_da, 1),(o1, r_mob, 3);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 2),(o2, r_uiux, 0),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 3),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 2),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q9 ───────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Ketika belajar hal baru, kamu paling suka...?', 9)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Mengikuti tutorial visual, Figma, dan design patterns', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Kursus statistik, pandas, atau data wrangling', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Membaca dokumentasi framework backend dan pola arsitektur', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Build sample app dan ikuti forum developer Android atau iOS', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 3),(o1, r_be, 0),(o1, r_uiux, 5),(o1, r_da, 1),(o1, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 1),(o2, r_uiux, 0),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 1),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 2),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 3),(o4, r_be, 1),(o4, r_uiux, 1),(o4, r_da, 0),(o4, r_mob, 5);

-- ── Q10 ──────────────────────────────────────────────────────────────────────
INSERT INTO quiz_questions (question_text, order_index) VALUES
  ('Kolaborasi yang paling kamu nikmati adalah...?', 10)
  RETURNING id INTO q;

INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Kerja bareng designer untuk polish antarmuka produk', 1) RETURNING id INTO o1;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Diskusi dengan data scientist soal model analitik', 2) RETURNING id INTO o2;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Review schema database dan API contract bersama tim', 3) RETURNING id INTO o3;
INSERT INTO quiz_options (question_id, option_text, order_index) VALUES (q, 'Test di berbagai device bersama QA dan tim mobile', 4) RETURNING id INTO o4;

INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o1, r_fe, 5),(o1, r_be, 1),(o1, r_uiux, 4),(o1, r_da, 0),(o1, r_mob, 3);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o2, r_fe, 0),(o2, r_be, 2),(o2, r_uiux, 1),(o2, r_da, 5),(o2, r_mob, 0);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o3, r_fe, 2),(o3, r_be, 5),(o3, r_uiux, 0),(o3, r_da, 3),(o3, r_mob, 2);
INSERT INTO option_role_weights (option_id, role_id, weight) VALUES (o4, r_fe, 2),(o4, r_be, 1),(o4, r_uiux, 2),(o4, r_da, 0),(o4, r_mob, 5);

END $$;
