DO $$
DECLARE
  r_fe UUID := '00000001-0000-0000-0000-000000000001';
  r_be UUID := '00000001-0000-0000-0000-000000000002';

  m UUID; s UUID;
BEGIN

-- ══════════════════════════════════════════════════════════════════════════════
-- FRONTEND MODULES
-- ══════════════════════════════════════════════════════════════════════════════

-- ── Module 1: HTML & CSS Dasar ────────────────────────────────────────────────
INSERT INTO learning_modules (id, role_id, title, description, order_index)
VALUES (gen_random_uuid(), r_fe,
        'HTML & CSS Dasar',
        'Pelajari fondasi web development: struktur dokumen HTML dan styling dengan CSS.',
        1)
RETURNING id INTO m;

-- Sub 1: Pengenalan HTML
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Pengenalan HTML', E'# Pengenalan HTML

HTML (HyperText Markup Language) adalah bahasa markah standar yang digunakan untuk membuat halaman web. Setiap halaman web yang kamu kunjungi dibangun menggunakan HTML sebagai fondasi strukturnya.

## Struktur Dasar HTML

Setiap dokumen HTML memiliki struktur dasar yang harus diikuti:

```html
<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Halaman Pertamaku</title>
</head>
<body>
    <h1>Halo Dunia!</h1>
    <p>Ini adalah paragraf pertamaku.</p>
</body>
</html>
```

## Elemen dan Tag

HTML menggunakan **tag** untuk mendefinisikan elemen. Tag biasanya berpasangan: tag pembuka `<p>` dan tag penutup `</p>`. Beberapa elemen penting:

- `<h1>` sampai `<h6>` — heading dengan level berbeda
- `<p>` — paragraf teks
- `<a href="...">` — hyperlink
- `<img src="..." alt="...">` — gambar
- `<ul>`, `<ol>`, `<li>` — daftar (list)
- `<div>` dan `<span>` — container generik

## Atribut HTML

Atribut memberikan informasi tambahan pada elemen. Contoh: `<a href="https://example.com" target="_blank">Klik di sini</a>`. Atribut `href` menentukan URL tujuan, sedangkan `target="_blank"` membuka link di tab baru.

## Semantic HTML

HTML5 memperkenalkan elemen semantik seperti `<header>`, `<nav>`, `<main>`, `<article>`, `<section>`, dan `<footer>`. Elemen ini membantu mesin pencari dan screen reader memahami struktur konten halaman dengan lebih baik.', 1, 15);

-- Sub 2: Selektor CSS
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Selektor CSS', E'# Selektor CSS

CSS (Cascading Style Sheets) digunakan untuk mengatur tampilan elemen HTML. Selektor adalah cara kita menunjuk elemen mana yang ingin di-style.

## Jenis-Jenis Selektor

### Selektor Elemen
Menargetkan semua elemen dengan tag tertentu:

```css
p {
    color: #333;
    font-size: 16px;
    line-height: 1.6;
}
```

### Selektor Class
Menggunakan titik (.) dan bisa dipakai berulang kali:

```css
.highlight {
    background-color: #fef08a;
    padding: 4px 8px;
    border-radius: 4px;
}
```

### Selektor ID
Menggunakan tanda pagar (#), unik per halaman:

```css
#main-header {
    background: linear-gradient(135deg, #667eea, #764ba2);
    color: white;
    padding: 20px;
}
```

## Specificity (Spesifisitas)

CSS memiliki aturan prioritas: inline style > ID > class > elemen. Ketika dua aturan konflik, yang lebih spesifik menang. Memahami specificity sangat penting untuk menghindari bug styling yang membingungkan.

## Pseudo-class dan Pseudo-element

- `:hover` — saat kursor berada di atas elemen
- `:first-child` — elemen pertama dalam parent
- `::before` dan `::after` — menyisipkan konten dekoratif
- `:focus` — saat elemen mendapat fokus keyboard

```css
.btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
```

## Combinator

Combinator menggabungkan beberapa selektor: descendant (spasi), child (`>`), adjacent sibling (`+`), dan general sibling (`~`).', 2, 15);

-- Sub 3: Flexbox dan Grid
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Flexbox dan Grid', E'# Flexbox dan Grid

Modern CSS layout menggunakan dua sistem utama: Flexbox untuk layout satu dimensi dan Grid untuk layout dua dimensi.

## Flexbox

Flexbox (Flexible Box Layout) ideal untuk menyusun elemen dalam satu baris atau kolom:

```css
.container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
}
```

### Properti Container Flex
- `flex-direction` — arah elemen: `row`, `column`, `row-reverse`
- `justify-content` — distribusi horizontal: `center`, `space-between`, `flex-end`
- `align-items` — alignment vertikal: `center`, `stretch`, `baseline`
- `flex-wrap` — apakah item boleh pindah baris: `wrap`, `nowrap`

### Properti Item Flex
- `flex-grow` — seberapa banyak item boleh membesar
- `flex-shrink` — seberapa banyak item boleh mengecil
- `flex-basis` — ukuran awal item sebelum distribusi ruang

## CSS Grid

Grid layout memungkinkan pengaturan baris DAN kolom secara bersamaan:

```css
.grid-layout {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-template-rows: auto;
    gap: 24px;
    padding: 20px;
}

.grid-item-featured {
    grid-column: span 2;
    grid-row: span 2;
}
```

## Kapan Menggunakan Flexbox vs Grid?

Gunakan **Flexbox** ketika kamu bekerja dengan layout satu dimensi — misalnya navigation bar, daftar kartu horizontal, atau centering konten. Gunakan **Grid** ketika kamu membutuhkan kontrol atas baris dan kolom — misalnya layout halaman utama, gallery foto, atau dashboard.

## Responsive Design

Kombinasikan Flexbox/Grid dengan media queries untuk layout yang responsif di berbagai ukuran layar:

```css
@media (max-width: 768px) {
    .grid-layout {
        grid-template-columns: 1fr;
    }
}
```', 3, 20);

-- ── Module 2: JavaScript Fundamental ─────────────────────────────────────────
INSERT INTO learning_modules (id, role_id, title, description, order_index)
VALUES (gen_random_uuid(), r_fe,
        'JavaScript Fundamental',
        'Kuasai dasar-dasar JavaScript: variabel, fungsi, dan manipulasi DOM.',
        2)
RETURNING id INTO m;

-- Sub 1: Variabel dan Tipe Data
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Variabel dan Tipe Data', E'# Variabel dan Tipe Data

JavaScript memiliki tiga cara mendeklarasikan variabel: `var`, `let`, dan `const`. Dalam pengembangan modern, kita lebih sering menggunakan `let` dan `const`.

## Deklarasi Variabel

```javascript
const nama = "Budi";        // tidak bisa di-reassign
let umur = 25;               // bisa di-reassign
var kota = "Jakarta";        // cara lama, hindari penggunaan
```

### Perbedaan let vs const vs var
- `const` — nilai tidak bisa diubah setelah inisialisasi, wajib diberi nilai saat deklarasi
- `let` — nilai bisa diubah, memiliki block scope
- `var` — memiliki function scope, bisa menyebabkan bug yang sulit dilacak

## Tipe Data Primitif

JavaScript memiliki 7 tipe data primitif:

```javascript
// String
const pesan = "Halo dunia!";

// Number (integer dan float)
const harga = 49999.99;

// Boolean
const aktif = true;

// Undefined
let belumDiisi;   // undefined

// Null
const kosong = null;

// BigInt
const angkaBesar = 9007199254740991n;

// Symbol
const id = Symbol("unik");
```

## Tipe Data Referensi

```javascript
// Object
const mahasiswa = {
    nama: "Andi",
    jurusan: "Informatika",
    semester: 6
};

// Array
const nilai = [85, 90, 78, 92];
```

## Type Checking

Gunakan `typeof` untuk mengecek tipe data:

```javascript
typeof "hello"     // "string"
typeof 42          // "number"
typeof true        // "boolean"
typeof undefined   // "undefined"
typeof null        // "object" (bug historis JS)
typeof {}          // "object"
```

## Template Literals

ES6 memperkenalkan template literals menggunakan backtick untuk interpolasi string:

```javascript
const nama = "Andi";
const pesan = `Halo, ${nama}! Selamat datang.`;
```', 1, 15);

-- Sub 2: Function dan Scope
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Function dan Scope', E'# Function dan Scope

Fungsi adalah blok kode yang dapat dipanggil ulang. Memahami fungsi dan scope sangat fundamental dalam JavaScript.

## Deklarasi Fungsi

Ada beberapa cara membuat fungsi di JavaScript:

```javascript
// Function Declaration
function tambah(a, b) {
    return a + b;
}

// Function Expression
const kali = function(a, b) {
    return a * b;
};

// Arrow Function (ES6)
const kurang = (a, b) => a - b;

// Arrow Function dengan body
const bagi = (a, b) => {
    if (b === 0) throw new Error("Tidak bisa bagi dengan nol");
    return a / b;
};
```

## Parameter dan Arguments

```javascript
// Default parameter
function sapa(nama = "Teman") {
    return `Halo, ${nama}!`;
}

// Rest parameter
function jumlahkan(...angka) {
    return angka.reduce((total, n) => total + n, 0);
}

jumlahkan(1, 2, 3, 4); // 10
```

## Scope

Scope menentukan di mana variabel bisa diakses:

### Global Scope
Variabel yang dideklarasikan di luar fungsi manapun.

### Function Scope
Variabel dengan `var` hanya bisa diakses dalam fungsi tempatnya dideklarasikan.

### Block Scope
Variabel dengan `let` dan `const` hanya bisa diakses dalam blok `{}` tempatnya:

```javascript
if (true) {
    let x = 10;     // hanya ada di dalam if
    const y = 20;   // hanya ada di dalam if
}
// x dan y tidak bisa diakses di sini
```

## Closure

Closure terjadi ketika fungsi mengingat variabel dari scope luarnya:

```javascript
function buatCounter() {
    let count = 0;
    return {
        tambah: () => ++count,
        nilai: () => count
    };
}

const counter = buatCounter();
counter.tambah(); // 1
counter.tambah(); // 2
counter.nilai();  // 2
```

Closure sangat berguna untuk data encapsulation dan factory pattern.', 2, 20);

-- Sub 3: DOM Manipulation
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'DOM Manipulation', E'# DOM Manipulation

DOM (Document Object Model) adalah representasi halaman web sebagai pohon objek. JavaScript dapat mengakses dan memanipulasi DOM untuk membuat halaman interaktif.

## Mengakses Elemen

```javascript
// By ID
const header = document.getElementById("main-header");

// By class name
const items = document.getElementsByClassName("menu-item");

// Query selector (CSS selector)
const btn = document.querySelector(".btn-primary");
const allCards = document.querySelectorAll(".card");
```

## Memodifikasi Konten

```javascript
// Mengubah teks
header.textContent = "Judul Baru";

// Mengubah HTML
header.innerHTML = "<span>Judul</span> <em>Baru</em>";

// Mengubah atribut
const link = document.querySelector("a");
link.setAttribute("href", "https://example.com");
link.classList.add("active");
link.classList.toggle("hidden");
```

## Mengubah Style

```javascript
const box = document.querySelector(".box");
box.style.backgroundColor = "#3b82f6";
box.style.padding = "20px";
box.style.borderRadius = "8px";
box.style.transform = "translateY(-4px)";
```

## Event Handling

Event listener memungkinkan kita merespons interaksi pengguna:

```javascript
const button = document.querySelector("#submit-btn");

button.addEventListener("click", function(event) {
    event.preventDefault();
    console.log("Tombol diklik!");
});

// Dengan arrow function
button.addEventListener("click", (e) => {
    e.target.textContent = "Sudah diklik!";
});
```

### Event yang Sering Digunakan
- `click` — klik mouse
- `input` — perubahan nilai input
- `submit` — form dikirim
- `keydown` / `keyup` — keyboard
- `mouseover` / `mouseout` — hover

## Membuat dan Menghapus Elemen

```javascript
// Membuat elemen baru
const card = document.createElement("div");
card.className = "card";
card.innerHTML = `
    <h3>Judul Card</h3>
    <p>Deskripsi card.</p>
`;

// Menambahkan ke DOM
document.querySelector(".container").appendChild(card);

// Menghapus elemen
card.remove();
```

DOM manipulation adalah dasar dari semua framework frontend modern seperti React, Vue, dan Angular.', 3, 20);

-- ══════════════════════════════════════════════════════════════════════════════
-- BACKEND MODULES
-- ══════════════════════════════════════════════════════════════════════════════

-- ── Module 1: Dasar HTTP & REST ───────────────────────────────────────────────
INSERT INTO learning_modules (id, role_id, title, description, order_index)
VALUES (gen_random_uuid(), r_be,
        'Dasar HTTP & REST',
        'Pahami protokol HTTP, metode request, dan prinsip desain RESTful API.',
        1)
RETURNING id INTO m;

-- Sub 1: Protokol HTTP
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Protokol HTTP', E'# Protokol HTTP

HTTP (HyperText Transfer Protocol) adalah protokol komunikasi yang menjadi fondasi pertukaran data di World Wide Web. Sebagai backend developer, memahami HTTP secara mendalam sangatlah penting.

## Cara Kerja HTTP

HTTP bekerja dengan model **request-response**: klien mengirim request ke server, lalu server memproses dan mengirim response kembali.

```
Client  ──── Request  ────>  Server
Client  <─── Response ─────  Server
```

## HTTP Methods

Setiap request memiliki method yang menunjukkan aksi yang diinginkan:

| Method   | Fungsi                    | Idempotent |
|----------|---------------------------|------------|
| `GET`    | Mengambil data            | Ya         |
| `POST`   | Membuat data baru         | Tidak      |
| `PUT`    | Mengganti data sepenuhnya | Ya         |
| `PATCH`  | Mengubah sebagian data    | Ya         |
| `DELETE` | Menghapus data            | Ya         |

## HTTP Status Codes

Status code menginformasikan hasil dari request:

- **2xx Success**: `200 OK`, `201 Created`, `204 No Content`
- **3xx Redirection**: `301 Moved Permanently`, `304 Not Modified`
- **4xx Client Error**: `400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`
- **5xx Server Error**: `500 Internal Server Error`, `502 Bad Gateway`

## Headers

HTTP headers membawa metadata tambahan:

```
Content-Type: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
Accept: application/json
Cache-Control: no-cache
```

Headers penting untuk autentikasi, content negotiation, dan caching.', 1, 15);

-- Sub 2: RESTful API Design
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'RESTful API Design', E'# RESTful API Design

REST (Representational State Transfer) adalah gaya arsitektur untuk mendesain API yang konsisten, scalable, dan mudah dipahami.

## Prinsip REST

1. **Stateless** — setiap request independen, server tidak menyimpan state klien
2. **Resource-based** — URL merepresentasikan resource (noun), bukan action (verb)
3. **Uniform Interface** — gunakan HTTP methods standar secara konsisten
4. **JSON sebagai format** — response dan request body dalam format JSON

## Desain URL yang Baik

```
GET    /api/users          → Daftar semua user
GET    /api/users/123      → Detail user dengan ID 123
POST   /api/users          → Buat user baru
PUT    /api/users/123      → Update user 123
DELETE /api/users/123      → Hapus user 123

GET    /api/users/123/posts → Semua post milik user 123
```

### Aturan Penamaan
- Gunakan **noun** (kata benda), bukan verb: `/users` bukan `/getUsers`
- Gunakan **plural**: `/users` bukan `/user`
- Gunakan **kebab-case** untuk multi-kata: `/user-profiles`
- Gunakan **query parameter** untuk filtering: `/users?role=admin&active=true`

## Response Format Konsisten

```json
{
    "success": true,
    "data": {
        "id": "abc-123",
        "name": "Budi",
        "email": "budi@example.com"
    }
}
```

Untuk error:

```json
{
    "success": false,
    "error": {
        "code": "NOT_FOUND",
        "message": "User dengan ID tersebut tidak ditemukan"
    }
}
```

## Pagination

Untuk endpoint yang mengembalikan list, implementasikan pagination:

```
GET /api/posts?page=2&per_page=20
```

Response harus menyertakan metadata pagination agar client tahu total halaman yang tersedia.', 2, 20);

-- ── Module 2: Database Relasional ─────────────────────────────────────────────
INSERT INTO learning_modules (id, role_id, title, description, order_index)
VALUES (gen_random_uuid(), r_be,
        'Database Relasional',
        'Kuasai konsep database relasional, SQL, dan desain skema yang efisien.',
        2)
RETURNING id INTO m;

-- Sub 1: Konsep Database dan SQL Dasar
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Konsep Database dan SQL Dasar', E'# Konsep Database dan SQL Dasar

Database relasional menyimpan data dalam tabel-tabel yang saling berelasi. PostgreSQL, MySQL, dan SQLite adalah contoh database relasional yang populer.

## Struktur Database

- **Database** — kumpulan tabel yang terorganisir
- **Tabel** — menyimpan data dalam baris dan kolom
- **Kolom** — mendefinisikan tipe data (VARCHAR, INT, BOOLEAN, dll)
- **Baris** — satu record/entri data

## Perintah SQL Dasar

### CREATE TABLE

```sql
CREATE TABLE mahasiswa (
    id          SERIAL PRIMARY KEY,
    nama        VARCHAR(100) NOT NULL,
    email       VARCHAR(255) UNIQUE NOT NULL,
    jurusan     VARCHAR(100),
    semester    INT DEFAULT 1,
    created_at  TIMESTAMP DEFAULT NOW()
);
```

### INSERT

```sql
INSERT INTO mahasiswa (nama, email, jurusan, semester)
VALUES (''Andi'', ''andi@univ.ac.id'', ''Informatika'', 6);
```

### SELECT

```sql
-- Ambil semua data
SELECT * FROM mahasiswa;

-- Dengan filter
SELECT nama, jurusan FROM mahasiswa
WHERE semester >= 5
ORDER BY nama ASC;

-- Dengan aggregate
SELECT jurusan, COUNT(*) as total
FROM mahasiswa
GROUP BY jurusan
HAVING COUNT(*) > 10;
```

### UPDATE dan DELETE

```sql
UPDATE mahasiswa SET semester = 7 WHERE id = 1;
DELETE FROM mahasiswa WHERE id = 1;
```

## Tipe Data Penting

| Tipe        | Fungsi                  |
|-------------|-------------------------|
| `VARCHAR`   | Teks dengan batas       |
| `TEXT`      | Teks tanpa batas        |
| `INT`       | Bilangan bulat          |
| `BOOLEAN`   | True/False              |
| `TIMESTAMP` | Tanggal dan waktu       |
| `UUID`      | ID unik universal       |
| `JSONB`     | Data JSON (PostgreSQL)  |', 1, 20);

-- Sub 2: Relasi dan JOIN
INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
VALUES (m, 'Relasi dan JOIN', E'# Relasi dan JOIN

Kekuatan database relasional terletak pada kemampuan menghubungkan tabel-tabel yang berbeda melalui foreign key dan operasi JOIN.

## Jenis Relasi

### One-to-Many (1:N)
Satu baris di tabel A berhubungan dengan banyak baris di tabel B:

```sql
CREATE TABLE penulis (
    id   SERIAL PRIMARY KEY,
    nama VARCHAR(100) NOT NULL
);

CREATE TABLE buku (
    id         SERIAL PRIMARY KEY,
    judul      VARCHAR(200) NOT NULL,
    penulis_id INT REFERENCES penulis(id)
);
```

### Many-to-Many (M:N)
Menggunakan tabel penghubung (junction table):

```sql
CREATE TABLE mahasiswa_matkul (
    mahasiswa_id INT REFERENCES mahasiswa(id),
    matkul_id    INT REFERENCES matkul(id),
    PRIMARY KEY (mahasiswa_id, matkul_id)
);
```

## Operasi JOIN

### INNER JOIN
Hanya menampilkan data yang cocok di kedua tabel:

```sql
SELECT b.judul, p.nama AS penulis
FROM buku b
INNER JOIN penulis p ON b.penulis_id = p.id;
```

### LEFT JOIN
Menampilkan semua data dari tabel kiri, meskipun tidak ada pasangan:

```sql
SELECT p.nama, COUNT(b.id) AS total_buku
FROM penulis p
LEFT JOIN buku b ON b.penulis_id = p.id
GROUP BY p.nama;
```

### Subquery

```sql
SELECT nama FROM mahasiswa
WHERE id IN (
    SELECT mahasiswa_id FROM mahasiswa_matkul
    WHERE matkul_id = 5
);
```

## Indexing

Index mempercepat query pada kolom yang sering di-filter atau di-join:

```sql
CREATE INDEX idx_buku_penulis ON buku (penulis_id);
CREATE INDEX idx_mahasiswa_email ON mahasiswa (email);
```

Tanpa index, database harus melakukan full table scan yang lambat pada tabel besar. Namun, terlalu banyak index memperlambat operasi INSERT dan UPDATE.

## Transaction

Transaction memastikan serangkaian operasi dijalankan secara atomik:

```sql
BEGIN;
UPDATE rekening SET saldo = saldo - 100000 WHERE id = 1;
UPDATE rekening SET saldo = saldo + 100000 WHERE id = 2;
COMMIT;
```

Jika salah satu gagal, semua perubahan dibatalkan dengan `ROLLBACK`.', 2, 25);

END $$;
