-- Seed: mini quizzes (per submaterial, 3 questions each) +
--       final quizzes (per module, 5 questions each).
--
-- Correct answer is ALWAYS option order_index=2 (the second option).
-- Submitting all order_index=1 options → score=0% (guaranteed fail).
-- Submitting all order_index=2 options → score=100% (guaranteed pass).
-- This makes integration tests deterministic without fixed UUIDs.
DO $$
DECLARE
    mod_id UUID;
    sub_id UUID;
    q      UUID;
BEGIN

-- ══════════════════════════════════════════════════════════════════════════════
-- FRONTEND — Module 1: HTML & CSS Dasar
-- ══════════════════════════════════════════════════════════════════════════════

SELECT id INTO mod_id FROM learning_modules WHERE title = 'HTML & CSS Dasar';

-- ── Sub 1: Pengenalan HTML ─────────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Pengenalan HTML';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Apa kepanjangan dari HTML?', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Hyper Transfer Markup Language', false, 1),
    (q, 'HyperText Markup Language',      true,  2),
    (q, 'Hyper Text Making Language',     false, 3),
    (q, 'High Text Markup Language',      false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Tag HTML yang digunakan untuk paragraf adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '<div>',  false, 1),
    (q, '<p>',    true,  2),
    (q, '<span>', false, 3),
    (q, '<text>', false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Elemen <title> harus diletakkan di dalam tag...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '<body>',   false, 1),
    (q, '<head>',   true,  2),
    (q, '<html>',   false, 3),
    (q, '<header>', false, 4);

-- ── Sub 2: Selektor CSS ────────────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Selektor CSS';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Selektor CSS mana yang menggunakan tanda titik (.)?', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Selektor elemen', false, 1),
    (q, 'Selektor class',  true,  2),
    (q, 'Selektor ID',     false, 3),
    (q, 'Selektor pseudo', false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Pseudo-class yang aktif saat kursor berada di atas elemen adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, ':focus',  false, 1),
    (q, ':hover',  true,  2),
    (q, ':active', false, 3),
    (q, ':visited',false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Specificity CSS dari tertinggi ke terendah adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Elemen > Class > ID > Inline', false, 1),
    (q, 'Inline > ID > Class > Elemen', true,  2),
    (q, 'ID > Inline > Class > Elemen', false, 3),
    (q, 'Class > ID > Inline > Elemen', false, 4);

-- ── Sub 3: Flexbox dan Grid ────────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Flexbox dan Grid';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Properti CSS untuk mengaktifkan Flexbox pada container adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'display: block',   false, 1),
    (q, 'display: flex',    true,  2),
    (q, 'display: inline',  false, 3),
    (q, 'display: grid',    false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Properti Flexbox untuk mengatur distribusi item secara horizontal adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'align-items',       false, 1),
    (q, 'justify-content',   true,  2),
    (q, 'flex-direction',    false, 3),
    (q, 'align-content',     false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Nilai CSS Grid untuk membuat 3 kolom sama lebar adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'grid-template-columns: 33% 33% 33%',      false, 1),
    (q, 'grid-template-columns: repeat(3, 1fr)',    true,  2),
    (q, 'grid-template-columns: 3 auto',            false, 3),
    (q, 'grid-columns: 3',                          false, 4);

-- ── Final Quiz: HTML & CSS Dasar (5 questions) ────────────────────────────

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Elemen HTML5 semantik yang digunakan untuk konten utama halaman adalah...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '<div id="main">',    false, 1),
    (q, '<main>',             true,  2),
    (q, '<content>',          false, 3),
    (q, '<section id="main">',false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Selektor CSS dengan specificity paling tinggi adalah...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Selektor elemen (p)',             false, 1),
    (q, 'Inline style (style="...")',      true,  2),
    (q, 'Selektor class (.class)',         false, 3),
    (q, 'Selektor ID (#id)',               false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Properti Flexbox yang menentukan arah sumbu utama layout adalah...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'justify-content',  false, 1),
    (q, 'flex-direction',   true,  2),
    (q, 'align-items',      false, 3),
    (q, 'flex-wrap',        false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Tag HTML yang benar untuk membuat hyperlink adalah...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '<link href="url">',  false, 1),
    (q, '<a href="url">',     true,  2),
    (q, '<url href="url">',   false, 3),
    (q, '<nav href="url">',   false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Atribut HTML yang wajib ada pada tag <img> untuk aksesibilitas adalah...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'src',   false, 1),
    (q, 'alt',   true,  2),
    (q, 'title', false, 3),
    (q, 'id',    false, 4);

-- ══════════════════════════════════════════════════════════════════════════════
-- FRONTEND — Module 2: JavaScript Fundamental
-- ══════════════════════════════════════════════════════════════════════════════

SELECT id INTO mod_id FROM learning_modules WHERE title = 'JavaScript Fundamental';

-- ── Sub 1: Variabel dan Tipe Data ─────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Variabel dan Tipe Data';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Keyword JavaScript mana yang tidak bisa di-reassign setelah inisialisasi?', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'var',   false, 1),
    (q, 'const', true,  2),
    (q, 'let',   false, 3),
    (q, 'set',   false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Apa hasil dari typeof null di JavaScript?', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '"null"',      false, 1),
    (q, '"object"',    true,  2),
    (q, '"undefined"', false, 3),
    (q, '"boolean"',   false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Template literal di JavaScript menggunakan karakter...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Tanda kutip tunggal ('')',  false, 1),
    (q, 'Backtick (`)',              true,  2),
    (q, 'Tanda kutip ganda (")',     false, 3),
    (q, 'Tanda pagar (#)',           false, 4);

-- ── Sub 2: Function dan Scope ─────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Function dan Scope';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Arrow function di JavaScript diperkenalkan pada versi...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'ES5', false, 1),
    (q, 'ES6', true,  2),
    (q, 'ES7', false, 3),
    (q, 'ES8', false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Closure dalam JavaScript adalah ketika sebuah fungsi...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Tidak memiliki return value',                              false, 1),
    (q, 'Mengingat variabel dari scope luar tempat ia dideklarasikan', true, 2),
    (q, 'Dipanggil secara rekursif',                               false, 3),
    (q, 'Memiliki parameter default',                              false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Variabel yang dideklarasikan dengan let memiliki...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Function scope',  false, 1),
    (q, 'Block scope',     true,  2),
    (q, 'Global scope',    false, 3),
    (q, 'Module scope',    false, 4);

-- ── Sub 3: DOM Manipulation ────────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'DOM Manipulation';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Method JavaScript untuk memilih elemen berdasarkan CSS selector adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'getElementById',    false, 1),
    (q, 'querySelector',     true,  2),
    (q, 'getElement',        false, 3),
    (q, 'findElement',       false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Event yang digunakan untuk mendeteksi klik mouse adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'mousedown', false, 1),
    (q, 'click',     true,  2),
    (q, 'press',     false, 3),
    (q, 'tap',       false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Property DOM yang mengubah konten teks elemen (tanpa parsing HTML) adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'innerHTML',    false, 1),
    (q, 'textContent',  true,  2),
    (q, 'innerText',    false, 3),
    (q, 'nodeValue',    false, 4);

-- ── Final Quiz: JavaScript Fundamental (5 questions) ─────────────────────

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Perbedaan utama antara let dan var di JavaScript adalah...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'let hanya bisa digunakan di dalam fungsi',                      false, 1),
    (q, 'let memiliki block scope sedangkan var memiliki function scope', true,  2),
    (q, 'var tidak bisa di-reassign sedangkan let bisa',                 false, 3),
    (q, 'Tidak ada perbedaan, keduanya sama',                            false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Rest parameter dalam JavaScript ditandai dengan...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Tanda &',   false, 1),
    (q, 'Tanda ...',  true,  2),
    (q, 'Tanda *',   false, 3),
    (q, 'Tanda @',   false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Method addEventListener menerima berapa argumen minimal?', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '1 argumen (event type)',              false, 1),
    (q, '2 argumen (event type dan listener)', true,  2),
    (q, '3 argumen (event, listener, options)',false, 3),
    (q, '4 argumen',                           false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Untuk membuat elemen HTML baru di JavaScript digunakan...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'document.newElement()',   false, 1),
    (q, 'document.createElement()', true, 2),
    (q, 'document.addElement()',   false, 3),
    (q, 'document.makeElement()',  false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Tipe data yang BUKAN primitif di JavaScript adalah...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'String',  false, 1),
    (q, 'Object',  true,  2),
    (q, 'Boolean', false, 3),
    (q, 'Number',  false, 4);

-- ══════════════════════════════════════════════════════════════════════════════
-- BACKEND — Module 1: Dasar HTTP & REST
-- ══════════════════════════════════════════════════════════════════════════════

SELECT id INTO mod_id FROM learning_modules WHERE title = 'Dasar HTTP & REST';

-- ── Sub 1: Protokol HTTP ──────────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Protokol HTTP';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'HTTP method yang digunakan untuk mengambil data tanpa mengubah state server adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'POST',  false, 1),
    (q, 'GET',   true,  2),
    (q, 'PUT',   false, 3),
    (q, 'PATCH', false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'HTTP status code 201 berarti...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'OK',      false, 1),
    (q, 'Created', true,  2),
    (q, 'Accepted',false, 3),
    (q, 'Found',   false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Header HTTP yang digunakan untuk autentikasi Bearer token adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Content-Type',  false, 1),
    (q, 'Authorization', true,  2),
    (q, 'Accept',        false, 3),
    (q, 'Cookie',        false, 4);

-- ── Sub 2: RESTful API Design ──────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'RESTful API Design';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Prinsip REST yang menyatakan server tidak menyimpan state klien disebut...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Uniform Interface', false, 1),
    (q, 'Stateless',         true,  2),
    (q, 'Cacheable',         false, 3),
    (q, 'Layered System',    false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'URL RESTful yang benar untuk mengambil detail user dengan ID 5 adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '/api/getUser/5',   false, 1),
    (q, '/api/users/5',     true,  2),
    (q, '/api/user-detail?id=5', false, 3),
    (q, '/api/fetchUser/5', false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Format penamaan URL RESTful yang direkomendasikan untuk multi-kata adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'camelCase (/userProfiles)',  false, 1),
    (q, 'kebab-case (/user-profiles)', true, 2),
    (q, 'snake_case (/user_profiles)', false, 3),
    (q, 'PascalCase (/UserProfiles)', false, 4);

-- ── Final Quiz: Dasar HTTP & REST (5 questions) ───────────────────────────

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'HTTP method mana yang bersifat idempotent (menghasilkan efek sama jika dipanggil berkali-kali)?', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'POST',   false, 1),
    (q, 'PUT',    true,  2),
    (q, 'PATCH',  false, 3),
    (q, 'DELETE yang tidak pernah idempotent', false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Status code HTTP yang menunjukkan resource tidak ditemukan adalah...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '403',  false, 1),
    (q, '404',  true,  2),
    (q, '500',  false, 3),
    (q, '401',  false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Dalam desain REST, URL sebaiknya menggunakan...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Verb (kata kerja) seperti /getUsers',   false, 1),
    (q, 'Noun (kata benda) seperti /users',       true,  2),
    (q, 'Kata kerja + objek seperti /listUsers',  false, 3),
    (q, 'Kata kerja dalam path seperti /fetch',   false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Untuk filtering pada endpoint REST, gunakan...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Path parameter seperti /users/active',      false, 1),
    (q, 'Query parameter seperti /users?active=true', true, 2),
    (q, 'Request body pada GET method',               false, 3),
    (q, 'Header khusus X-Filter',                     false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Status code 401 Unauthorized berbeda dengan 403 Forbidden karena...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, '401 untuk server error, 403 untuk client error',               false, 1),
    (q, '401 berarti belum autentikasi, 403 berarti sudah autentikasi tapi tidak punya izin', true, 2),
    (q, '401 untuk resource tidak ditemukan, 403 untuk akses ditolak',  false, 3),
    (q, 'Keduanya berarti hal yang sama',                               false, 4);

-- ══════════════════════════════════════════════════════════════════════════════
-- BACKEND — Module 2: Database Relasional
-- ══════════════════════════════════════════════════════════════════════════════

SELECT id INTO mod_id FROM learning_modules WHERE title = 'Database Relasional';

-- ── Sub 1: Konsep Database dan SQL Dasar ──────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Konsep Database dan SQL Dasar';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Perintah SQL untuk mengambil semua data dari tabel "users" adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'FETCH ALL FROM users',  false, 1),
    (q, 'SELECT * FROM users',   true,  2),
    (q, 'GET ALL users',         false, 3),
    (q, 'READ users',            false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Tipe data SQL yang digunakan untuk menyimpan ID unik universal adalah...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'VARCHAR',  false, 1),
    (q, 'UUID',     true,  2),
    (q, 'INT',      false, 3),
    (q, 'BIGINT',   false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Klausa SQL untuk memfilter hasil GROUP BY berdasarkan kondisi aggregat adalah...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'WHERE',  false, 1),
    (q, 'HAVING', true,  2),
    (q, 'FILTER', false, 3),
    (q, 'WHEN',   false, 4);

-- ── Sub 2: Relasi dan JOIN ─────────────────────────────────────────────────

SELECT s.id INTO sub_id
FROM submaterials s WHERE s.module_id = mod_id AND s.title = 'Relasi dan JOIN';

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'JOIN yang menampilkan semua baris dari tabel kiri meskipun tidak ada pasangan di tabel kanan adalah...', 1) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'INNER JOIN', false, 1),
    (q, 'LEFT JOIN',  true,  2),
    (q, 'RIGHT JOIN', false, 3),
    (q, 'FULL JOIN',  false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Relasi Many-to-Many (M:N) dalam database relasional diimplementasikan dengan...', 2) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Foreign key langsung antara dua tabel',                   false, 1),
    (q, 'Tabel penghubung (junction table) dengan dua foreign key', true,  2),
    (q, 'Kolom JSON yang menyimpan array ID',                       false, 3),
    (q, 'Trigger database',                                         false, 4);

INSERT INTO submaterial_quizzes (submaterial_id, question, order_index) VALUES (sub_id, 'Index pada kolom database berguna untuk...', 3) RETURNING id INTO q;
INSERT INTO submaterial_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Mempercepat operasi INSERT dan UPDATE',   false, 1),
    (q, 'Mempercepat operasi SELECT (query/filter)', true, 2),
    (q, 'Memastikan keunikan data (UNIQUE)',        false, 3),
    (q, 'Menggabungkan dua tabel',                 false, 4);

-- ── Final Quiz: Database Relasional (5 questions) ─────────────────────────

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'ACID dalam konteks database transaction adalah singkatan dari...', 1) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Access, Control, Integrity, Durability',                    false, 1),
    (q, 'Atomicity, Consistency, Isolation, Durability',             true,  2),
    (q, 'Authentication, Consistency, Integrity, Data',              false, 3),
    (q, 'Atomicity, Caching, Indexing, Distribution',                false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'INNER JOIN mengembalikan...', 2) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Semua baris dari kedua tabel',                         false, 1),
    (q, 'Hanya baris yang memiliki pasangan di kedua tabel',    true,  2),
    (q, 'Semua baris dari tabel kiri',                         false, 3),
    (q, 'Semua baris dari tabel kanan',                        false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Constraint database yang memastikan nilai kolom selalu unik adalah...', 3) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'NOT NULL',    false, 1),
    (q, 'UNIQUE',      true,  2),
    (q, 'PRIMARY KEY', false, 3),
    (q, 'CHECK',       false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Dalam desain database, normalisasi bertujuan untuk...', 4) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'Mempercepat semua query',                                  false, 1),
    (q, 'Mengurangi redundansi data dan dependensi yang tidak perlu', true, 2),
    (q, 'Menambahkan lebih banyak index',                           false, 3),
    (q, 'Menggabungkan semua tabel menjadi satu',                   false, 4);

INSERT INTO module_quizzes (module_id, question, order_index) VALUES (mod_id, 'Perintah SQL untuk membatalkan semua perubahan dalam transaction adalah...', 5) RETURNING id INTO q;
INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES
    (q, 'CANCEL',   false, 1),
    (q, 'ROLLBACK', true,  2),
    (q, 'REVERT',   false, 3),
    (q, 'UNDO',     false, 4);

END $$;
