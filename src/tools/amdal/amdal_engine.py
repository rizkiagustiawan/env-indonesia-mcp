#!/usr/bin/env python3
"""AMDAL Document Generator Engine
Generates PDF documents for Indonesian Environmental Impact Assessment (AMDAL)
Ref: PermenLHK No. 5/2021, PermenLHK No. 6/2021
"""

import sys
import json
import os
from datetime import datetime

try:
    from fpdf import FPDF
except ImportError:
    print("ERROR: fpdf2 belum terinstall. Jalankan: pip install fpdf2")
    sys.exit(1)


class AMDALDocument(FPDF):
    """Base AMDAL PDF document with standard formatting."""

    def __init__(self, title="Dokumen AMDAL", project_name="", location=""):
        super().__init__()
        self.doc_title = title
        self.project_name = project_name
        self.location = location
        self.section_num = 0
        self.sub_num = 0
        self.set_auto_page_break(auto=True, margin=25)
        try:
            self.add_font('DejaVu', '', '/usr/share/fonts/TTF/DejaVuSans.ttf', uni=True)
            self.add_font('DejaVu', 'B', '/usr/share/fonts/TTF/DejaVuSans-Bold.ttf', uni=True)
            self.add_font('DejaVu', 'I', '/usr/share/fonts/TTF/DejaVuSans-Oblique.ttf', uni=True)
            self.default_font = 'DejaVu'
        except Exception:
            self.default_font = 'Helvetica'

    def sanitize_text(self, text):
        return text.replace('\u2014', '-').replace('\u2013', '-').replace('\u201c', '"').replace('\u201d', '"').replace('\u2018', "'").replace('\u2019', "'")

    def header(self):
        self.set_font(self.default_font, "B", 10)
        self.cell(0, 6, self.doc_title, border=0, align="L")
        self.cell(0, 6, self.project_name, border=0, align="R", new_x="LMARGIN", new_y="NEXT")
        self.line(10, self.get_y(), 200, self.get_y())
        self.ln(4)

    def footer(self):
        self.set_y(-20)
        self.line(10, self.get_y(), 200, self.get_y())
        self.ln(2)
        self.set_font(self.default_font, "I", 8)
        self.cell(0, 5, f"Halaman {self.page_no()}/{{nb}}", align="C", new_x="LMARGIN", new_y="NEXT")
        self.cell(0, 5, f"Digenerate oleh ZeroClaw Environmental AI | {datetime.now().strftime('%d %B %Y')}", align="C")

    def add_cover(self, doc_type, project_name, location, extra_lines=None):
        self.add_page()
        self.ln(40)
        self.set_font(self.default_font, "B", 22)
        self.cell(0, 15, doc_type, align="C", new_x="LMARGIN", new_y="NEXT")
        self.ln(10)
        self.set_font(self.default_font, "", 16)
        self.cell(0, 10, project_name, align="C", new_x="LMARGIN", new_y="NEXT")
        self.ln(5)
        self.set_font(self.default_font, "", 13)
        self.cell(0, 10, f"Lokasi: {location}", align="C", new_x="LMARGIN", new_y="NEXT")
        self.ln(10)
        if extra_lines:
            self.set_font(self.default_font, "", 11)
            for line in extra_lines:
                self.cell(0, 8, line, align="C", new_x="LMARGIN", new_y="NEXT")
        self.ln(20)
        self.set_font(self.default_font, "I", 10)
        self.cell(0, 8, f"Disusun berdasarkan PermenLHK No. 5 Tahun 2021", align="C", new_x="LMARGIN", new_y="NEXT")
        self.cell(0, 8, f"dan PermenLHK No. 6 Tahun 2021", align="C", new_x="LMARGIN", new_y="NEXT")
        self.ln(10)
        self.cell(0, 8, f"Tanggal: {datetime.now().strftime('%d %B %Y')}", align="C", new_x="LMARGIN", new_y="NEXT")

    def add_section(self, title):
        self.section_num += 1
        self.sub_num = 0
        self.ln(4)
        self.set_font(self.default_font, "B", 14)
        self.cell(0, 10, f"BAB {self.section_num}. {title}", new_x="LMARGIN", new_y="NEXT")
        self.ln(2)

    def add_subsection(self, title):
        self.sub_num += 1
        self.ln(2)
        self.set_font(self.default_font, "B", 12)
        self.cell(0, 8, f"{self.section_num}.{self.sub_num} {title}", new_x="LMARGIN", new_y="NEXT")
        self.ln(1)

    def add_paragraph(self, text):
        self.set_font(self.default_font, "", 10)
        self.multi_cell(0, 6, text)
        self.ln(2)

    def add_table(self, headers, rows, col_widths=None):
        if col_widths is None:
            n = len(headers)
            col_widths = [190 / n] * n
        # Header
        self.set_font(self.default_font, "B", 9)
        self.set_fill_color(51, 102, 153)
        self.set_text_color(255, 255, 255)
        for i, h in enumerate(headers):
            self.cell(col_widths[i], 8, h, border=1, fill=True, align="C")
        self.ln()
        # Rows
        self.set_font(self.default_font, "", 9)
        self.set_text_color(0, 0, 0)
        fill = False
        for row in rows:
            if fill:
                self.set_fill_color(230, 240, 250)
            else:
                self.set_fill_color(255, 255, 255)
            max_h = 8
            for i, cell in enumerate(row):
                self.cell(col_widths[i], max_h, str(cell)[:40], border=1, fill=True, align="L")
            self.ln()
            fill = not fill


def generate_ka_andal(project_name, location, project_type, rona_data, output_path):
    """Generate Kerangka Acuan ANDAL document."""
    pdf = AMDALDocument("KERANGKA ACUAN ANDAL", project_name, location)
    pdf.alias_nb_pages()
    pdf.add_cover("KERANGKA ACUAN\nANALISIS DAMPAK LINGKUNGAN HIDUP\n(KA-ANDAL)", project_name, location,
                  [f"Jenis Usaha/Kegiatan: {project_type}"])

    # BAB 1: Pendahuluan
    pdf.add_page()
    pdf.add_section("PENDAHULUAN")
    pdf.add_subsection("Latar Belakang")
    pdf.add_paragraph(
        f"Dokumen Kerangka Acuan Analisis Dampak Lingkungan Hidup (KA-ANDAL) ini disusun sebagai "
        f"panduan pelaksanaan studi ANDAL untuk rencana kegiatan {project_type} yang berlokasi di "
        f"{location}. Penyusunan KA-ANDAL ini mengacu pada Peraturan Menteri Lingkungan Hidup dan "
        f"Kehutanan Nomor 5 Tahun 2021 tentang Tata Laksana dan Pemeriksaan Dokumen Lingkungan Hidup "
        f"serta Penerbitan Persetujuan Lingkungan."
    )
    pdf.add_subsection("Tujuan dan Kegunaan Studi")
    pdf.add_paragraph(
        "Studi ANDAL bertujuan untuk:\n"
        "1. Mengidentifikasi rona lingkungan hidup awal di wilayah studi\n"
        "2. Memprakirakan dampak penting yang akan timbul dari rencana kegiatan\n"
        "3. Mengevaluasi dampak penting secara holistik\n"
        "4. Merumuskan arahan pengelolaan dan pemantauan lingkungan hidup"
    )
    pdf.add_subsection("Peraturan Perundang-undangan")
    pdf.add_paragraph(
        "1. UU No. 32 Tahun 2009 tentang Perlindungan dan Pengelolaan Lingkungan Hidup\n"
        "2. PP No. 22 Tahun 2021 tentang Penyelenggaraan Perlindungan dan Pengelolaan LH\n"
        "3. PermenLHK No. 5 Tahun 2021 tentang Tata Laksana Dokumen Lingkungan Hidup\n"
        "4. PermenLHK No. 4 Tahun 2021 tentang Daftar Usaha/Kegiatan Wajib AMDAL/UKL-UPL"
    )

    # BAB 2: Pelingkupan
    pdf.add_section("PELINGKUPAN")
    pdf.add_subsection("Deskripsi Rencana Kegiatan")
    pdf.add_paragraph(
        f"Rencana kegiatan {project_type} dengan nama proyek \"{project_name}\" berlokasi di {location}. "
        f"Kegiatan ini mencakup tahap pra-konstruksi, konstruksi, operasi, dan pasca-operasi."
    )
    pdf.add_subsection("Komponen Lingkungan yang Terkena Dampak")
    pdf.add_paragraph(
        "Komponen lingkungan yang berpotensi terkena dampak meliputi:\n"
        "a. Komponen Geofisik-Kimia: kualitas udara, kebisingan, kualitas air, tanah\n"
        "b. Komponen Biologi: flora darat, fauna darat, biota perairan\n"
        "c. Komponen Sosial Ekonomi Budaya: pendapatan, lapangan kerja, kesehatan masyarakat\n"
        "d. Komponen Kesehatan Masyarakat: sanitasi, penyakit terkait lingkungan"
    )

    # BAB 3: Metodologi
    pdf.add_section("METODOLOGI STUDI")
    pdf.add_subsection("Metode Pengumpulan Data")
    pdf.add_paragraph(
        "Pengumpulan data dilakukan melalui:\n"
        "1. Survei lapangan dan pengukuran langsung parameter lingkungan\n"
        "2. Wawancara dan kuesioner dengan masyarakat terdampak\n"
        "3. Pengambilan sampel (air, udara, tanah, biota)\n"
        "4. Analisis laboratorium\n"
        "5. Penginderaan jauh dan analisis spasial\n"
        "6. Studi literatur dan data sekunder"
    )
    pdf.add_subsection("Metode Prakiraan Dampak")
    pdf.add_paragraph(
        "Prakiraan dampak menggunakan metode:\n"
        "1. Model matematika (dispersi udara, penyebaran kebisingan)\n"
        "2. Analogi dengan kegiatan sejenis\n"
        "3. Penilaian ahli (professional judgement)\n"
        "4. Matriks interaksi Leopold"
    )

    # BAB 4: Rona Lingkungan Hidup Awal
    pdf.add_section("RONA LINGKUNGAN HIDUP AWAL")
    if rona_data:
        for component in rona_data:
            name = component.get("komponen", "Komponen")
            desc = component.get("deskripsi", "")
            kondisi = component.get("kondisi", "")
            pdf.add_subsection(name)
            if desc:
                pdf.add_paragraph(desc)
            if kondisi:
                pdf.add_paragraph(f"Kondisi eksisting: {kondisi}")
    else:
        pdf.add_paragraph("Data rona lingkungan hidup awal akan dikumpulkan pada tahap studi ANDAL.")

    # BAB 5: Dampak Penting Hipotetik
    pdf.add_section("DAMPAK PENTING HIPOTETIK")
    pdf.add_paragraph(
        "Dampak penting hipotetik diidentifikasi berdasarkan interaksi antara komponen rencana "
        "kegiatan dengan komponen lingkungan hidup, menggunakan kriteria dampak penting sesuai "
        "PP 22/2021 Pasal 14."
    )
    pdf.add_table(
        ["No", "Tahap", "Komponen Kegiatan", "Dampak Hipotetik", "Komponen LH"],
        [
            ["1", "Pra-Konstruksi", "Pembebasan lahan", "Keresahan masyarakat", "Sosial"],
            ["2", "Konstruksi", "Pembersihan lahan", "Penurunan kualitas udara", "Fisik-Kimia"],
            ["3", "Konstruksi", "Mobilisasi alat berat", "Peningkatan kebisingan", "Fisik-Kimia"],
            ["4", "Konstruksi", "Galian dan timbunan", "Erosi dan sedimentasi", "Fisik-Kimia"],
            ["5", "Operasi", "Operasional kegiatan", "Pencemaran air limbah", "Fisik-Kimia"],
            ["6", "Operasi", "Operasional kegiatan", "Gangguan biota", "Biologi"],
            ["7", "Pasca-Operasi", "Penutupan kegiatan", "Pemulihan lahan", "Fisik-Kimia"],
        ],
        [10, 30, 40, 50, 30]
    )

    # BAB 6: Batas Waktu dan Wilayah Studi
    pdf.add_section("BATAS WAKTU DAN WILAYAH STUDI")
    pdf.add_subsection("Batas Wilayah Studi")
    pdf.add_paragraph(
        "Batas wilayah studi ditentukan berdasarkan:\n"
        "1. Batas proyek (tapak kegiatan)\n"
        "2. Batas ekologis (DAS, ekosistem terkait)\n"
        "3. Batas sosial (wilayah administratif terdampak)\n"
        "4. Batas administratif (kecamatan/kabupaten terkait)"
    )
    pdf.add_subsection("Batas Waktu Studi")
    pdf.add_paragraph(
        "Studi ANDAL dilaksanakan dalam jangka waktu yang mencakup seluruh tahap kegiatan "
        "mulai dari pra-konstruksi hingga pasca-operasi, dengan estimasi waktu studi 6-12 bulan."
    )

    pdf.output(output_path)
    return f"SUCCESS: KA-ANDAL disimpan di {output_path}"


def generate_andal(project_name, location, impacts_data, output_path):
    """Generate ANDAL (Analisis Dampak Lingkungan) document."""
    pdf = AMDALDocument("ANALISIS DAMPAK LINGKUNGAN HIDUP", project_name, location)
    pdf.alias_nb_pages()
    pdf.add_cover("ANALISIS DAMPAK LINGKUNGAN HIDUP\n(ANDAL)", project_name, location)

    # BAB 1: Deskripsi Rona Lingkungan Hidup
    pdf.add_page()
    pdf.add_section("DESKRIPSI RONA LINGKUNGAN HIDUP")
    pdf.add_subsection("Komponen Geofisik-Kimia")
    pdf.add_paragraph(
        "Deskripsi kondisi awal komponen geofisik-kimia meliputi iklim dan kualitas udara, "
        "fisiografi dan geologi, hidrologi dan kualitas air permukaan, hidrogeologi dan kualitas "
        "air tanah, serta kondisi tanah dan penggunaan lahan di wilayah studi."
    )
    pdf.add_subsection("Komponen Biologi")
    pdf.add_paragraph(
        "Komponen biologi mencakup vegetasi darat (flora), satwa darat (fauna), biota perairan "
        "(plankton, benthos, nekton), serta ekosistem sensitif yang terdapat di wilayah studi "
        "seperti mangrove, terumbu karang, dan kawasan hutan."
    )
    pdf.add_subsection("Komponen Sosial Ekonomi Budaya")
    pdf.add_paragraph(
        "Kondisi sosial ekonomi budaya meliputi demografi, mata pencaharian, tingkat pendapatan, "
        "persepsi masyarakat, adat istiadat, dan kondisi kesehatan masyarakat di sekitar "
        "rencana kegiatan."
    )

    # BAB 2: Prakiraan Dampak Penting
    pdf.add_section("PRAKIRAAN DAMPAK PENTING")
    pdf.add_paragraph(
        "Prakiraan dampak penting dilakukan terhadap setiap dampak hipotetik yang telah "
        "diidentifikasi pada tahap pelingkupan. Prakiraan mencakup besaran dampak (magnitude), "
        "durasi, dan reversibilitas dampak."
    )

    if impacts_data:
        headers = ["No", "Dampak", "Magnitude", "Durasi", "Reversibilitas", "Sifat"]
        rows = []
        for i, impact in enumerate(impacts_data, 1):
            rows.append([
                str(i),
                impact.get("dampak", ""),
                str(impact.get("magnitude", "")),
                impact.get("durasi", ""),
                impact.get("reversibilitas", ""),
                impact.get("sifat", "negatif")
            ])
        pdf.add_table(headers, rows, [10, 55, 25, 30, 35, 25])

        # Narrative for each impact
        for i, impact in enumerate(impacts_data, 1):
            pdf.add_subsection(f"Dampak {i}: {impact.get('dampak', '')}")
            pdf.add_paragraph(
                f"Besaran dampak: {impact.get('magnitude', 'N/A')} ({impact.get('sifat', 'negatif')})\n"
                f"Durasi: {impact.get('durasi', 'N/A')}\n"
                f"Reversibilitas: {impact.get('reversibilitas', 'N/A')}\n"
                f"Deskripsi: {impact.get('deskripsi', 'Memerlukan kajian lebih lanjut.')}"
            )
    else:
        pdf.add_paragraph("Data prakiraan dampak akan disajikan setelah analisis lapangan selesai.")

    # BAB 3: Evaluasi Holistik
    pdf.add_section("EVALUASI HOLISTIK")
    pdf.add_paragraph(
        "Evaluasi secara holistik dilakukan untuk menilai kelayakan lingkungan hidup dari "
        "rencana kegiatan. Evaluasi ini mempertimbangkan interaksi dan keterkaitan antar "
        "dampak penting yang terjadi secara simultan."
    )
    pdf.add_subsection("Telaahan Terhadap Rencana Kegiatan")
    pdf.add_paragraph(
        "Secara keseluruhan, rencana kegiatan ini perlu memperhatikan aspek keberlanjutan "
        "lingkungan hidup. Dampak negatif yang timbul harus dapat dikelola melalui upaya "
        "pengelolaan dan pemantauan lingkungan hidup yang komprehensif sesuai dengan "
        "RKL-RPL yang disusun."
    )
    pdf.add_subsection("Keterkaitan Antar Dampak")
    pdf.add_paragraph(
        "Keterkaitan antar dampak penting menunjukkan bahwa perubahan pada satu komponen "
        "lingkungan dapat mempengaruhi komponen lainnya. Oleh karena itu, pengelolaan lingkungan "
        "harus dilakukan secara terpadu dan menyeluruh."
    )

    pdf.output(output_path)
    return f"SUCCESS: ANDAL disimpan di {output_path}"


def generate_rkl_rpl(project_name, location, management_data, output_path):
    """Generate RKL-RPL (Environmental Management and Monitoring Plan)."""
    pdf = AMDALDocument("RENCANA PENGELOLAAN & PEMANTAUAN LH", project_name, location)
    pdf.alias_nb_pages()
    pdf.add_cover("RENCANA PENGELOLAAN LINGKUNGAN HIDUP\nDAN\nRENCANA PEMANTAUAN LINGKUNGAN HIDUP\n(RKL-RPL)",
                  project_name, location)

    # RKL Section
    pdf.add_page()
    pdf.add_section("RENCANA PENGELOLAAN LINGKUNGAN HIDUP (RKL)")
    pdf.add_paragraph(
        "Rencana Pengelolaan Lingkungan Hidup (RKL) disusun sebagai upaya untuk mencegah, "
        "mengendalikan, dan menanggulangi dampak penting negatif serta mengembangkan dampak "
        "positif dari rencana kegiatan terhadap lingkungan hidup."
    )

    if management_data:
        headers = ["Dampak Penting", "Sumber Dampak", "Pengelolaan", "Pemantauan", "Institusi"]
        rows = []
        for item in management_data:
            rows.append([
                item.get("dampak", ""),
                item.get("sumber", ""),
                item.get("pengelolaan", ""),
                item.get("pemantauan", ""),
                item.get("institusi", "DLHK")
            ])
        pdf.add_table(headers, rows, [35, 30, 45, 40, 30])

        # Detailed management plans
        for i, item in enumerate(management_data, 1):
            pdf.add_subsection(f"Pengelolaan Dampak {i}: {item.get('dampak', '')}")
            pdf.add_paragraph(f"Sumber dampak: {item.get('sumber', 'N/A')}")
            pdf.add_paragraph(f"Bentuk pengelolaan: {item.get('pengelolaan', 'N/A')}")
            pdf.add_paragraph(f"Lokasi pengelolaan: {item.get('lokasi_kelola', location)}")
            pdf.add_paragraph(f"Periode pengelolaan: {item.get('periode', 'Selama kegiatan berlangsung')}")
    else:
        pdf.add_paragraph("Matriks pengelolaan lingkungan akan disusun berdasarkan hasil studi ANDAL.")

    # RPL Section
    pdf.add_section("RENCANA PEMANTAUAN LINGKUNGAN HIDUP (RPL)")
    pdf.add_paragraph(
        "Rencana Pemantauan Lingkungan Hidup (RPL) disusun untuk memantau efektivitas upaya "
        "pengelolaan lingkungan hidup yang telah dilaksanakan. Pemantauan dilakukan secara "
        "berkala dan hasilnya dilaporkan kepada instansi terkait."
    )

    if management_data:
        headers = ["Dampak", "Parameter Pantau", "Metode", "Frekuensi", "Lokasi"]
        rows = []
        for item in management_data:
            rows.append([
                item.get("dampak", ""),
                item.get("parameter_pantau", item.get("dampak", "")),
                item.get("metode_pantau", "Pengukuran langsung"),
                item.get("frekuensi", "Bulanan"),
                item.get("lokasi_pantau", location)
            ])
        pdf.add_table(headers, rows, [35, 35, 40, 30, 40])

    pdf.output(output_path)
    return f"SUCCESS: RKL-RPL disimpan di {output_path}"


def generate_ukl_upl(project_name, location, impacts_data, output_path):
    """Generate UKL-UPL (simplified environmental document)."""
    pdf = AMDALDocument("FORMULIR UKL-UPL", project_name, location)
    pdf.alias_nb_pages()
    pdf.add_cover("FORMULIR\nUPAYA PENGELOLAAN LINGKUNGAN HIDUP\nDAN\nUPAYA PEMANTAUAN LINGKUNGAN HIDUP\n(UKL-UPL)",
                  project_name, location,
                  ["Sesuai format PermenLHK No. 6 Tahun 2021"])

    # Identitas Pemrakarsa
    pdf.add_page()
    pdf.add_section("IDENTITAS PEMRAKARSA")
    pdf.add_table(
        ["Uraian", "Keterangan"],
        [
            ["Nama Usaha/Kegiatan", project_name],
            ["Lokasi", location],
            ["Skala/Besaran", "Sesuai spesifikasi teknis"],
            ["Nama Pemrakarsa", "[Nama Pemrakarsa]"],
            ["Alamat Pemrakarsa", "[Alamat]"],
            ["No. Telepon", "[Telepon]"],
            ["Penanggung Jawab", "[Nama PJ]"],
        ],
        [60, 120]
    )

    # Rencana Usaha/Kegiatan
    pdf.add_section("RENCANA USAHA DAN/ATAU KEGIATAN")
    pdf.add_paragraph(
        f"Rencana kegiatan \"{project_name}\" berlokasi di {location}. "
        f"Kegiatan meliputi tahap pra-konstruksi, konstruksi, operasi, dan pasca-operasi."
    )

    # Dampak dan Pengelolaan
    pdf.add_section("DAMPAK LINGKUNGAN YANG DITIMBULKAN DAN UPAYA PENGELOLAAN serta PEMANTAUAN LH")
    if impacts_data:
        for i, impact in enumerate(impacts_data, 1):
            pdf.add_subsection(f"Dampak {i}: {impact.get('dampak', '')}")
            pdf.add_table(
                ["Aspek", "Uraian"],
                [
                    ["Sumber dampak", impact.get("sumber", "")],
                    ["Jenis dampak", impact.get("dampak", "")],
                    ["Besaran dampak", str(impact.get("magnitude", ""))],
                    ["Upaya pengelolaan", impact.get("pengelolaan", "")],
                    ["Upaya pemantauan", impact.get("pemantauan", "")],
                    ["Institusi pengelola", impact.get("institusi", "DLHK Provinsi")],
                ],
                [50, 130]
            )
            pdf.ln(3)
    else:
        pdf.add_paragraph("Data dampak akan dilengkapi setelah survei lapangan.")

    # Komitmen Pengelolaan
    pdf.add_section("SURAT PERNYATAAN KESANGGUPAN PENGELOLAAN DAN PEMANTAUAN LH")
    pdf.add_paragraph(
        "Yang bertanda tangan di bawah ini menyatakan bersedia dan sanggup untuk melaksanakan "
        "upaya pengelolaan lingkungan hidup dan upaya pemantauan lingkungan hidup sebagaimana "
        "tercantum dalam dokumen UKL-UPL ini.\n\n"
        "Apabila di kemudian hari terdapat perubahan rencana kegiatan, maka saya akan "
        "mengajukan perubahan dokumen UKL-UPL sesuai peraturan yang berlaku."
    )
    pdf.ln(15)
    pdf.set_font(pdf.default_font, "", 10)
    pdf.cell(95, 8, f"{location}, {datetime.now().strftime('%d %B %Y')}", align="R")
    pdf.cell(95, 8, "", new_x="LMARGIN", new_y="NEXT")
    pdf.ln(20)
    pdf.cell(95, 8, "")
    pdf.cell(95, 8, "Pemrakarsa,", align="C", new_x="LMARGIN", new_y="NEXT")
    pdf.ln(20)
    pdf.cell(95, 8, "")
    pdf.cell(95, 8, "(_______________________)", align="C", new_x="LMARGIN", new_y="NEXT")

    pdf.output(output_path)
    return f"SUCCESS: UKL-UPL disimpan di {output_path}"


def generate_klhs(policy_name, daya_dukung_data, output_path):
    """Generate KLHS (Strategic Environmental Assessment)."""
    pdf = AMDALDocument("KAJIAN LINGKUNGAN HIDUP STRATEGIS", policy_name, "")
    pdf.alias_nb_pages()
    pdf.add_cover("KAJIAN LINGKUNGAN HIDUP STRATEGIS\n(KLHS)", policy_name, "",
                  ["Sesuai PP No. 46 Tahun 2016"])

    pdf.add_page()
    pdf.add_section("DAYA DUKUNG LINGKUNGAN HIDUP")
    pdf.add_paragraph(
        "Analisis daya dukung lingkungan hidup dilakukan untuk menentukan kemampuan lingkungan "
        "hidup dalam mendukung perikehidupan manusia, makhluk hidup lain, dan keseimbangan "
        "antar keduanya. Daya dukung meliputi kapasitas penyediaan (provisioning) dan kapasitas "
        "pengaturan (regulating) dari ekosistem."
    )

    if daya_dukung_data:
        for item in daya_dukung_data:
            pdf.add_subsection(item.get("aspek", ""))
            pdf.add_paragraph(f"Kondisi: {item.get('kondisi', 'N/A')}")
            pdf.add_paragraph(f"Kapasitas: {item.get('kapasitas', 'N/A')}")
            pdf.add_paragraph(f"Tekanan: {item.get('tekanan', 'N/A')}")
            status = item.get("status", "sedang")
            if status == "kritis":
                pdf.add_paragraph("STATUS: KRITIS - Memerlukan intervensi segera")
            elif status == "terlampaui":
                pdf.add_paragraph("STATUS: TERLAMPAUI - Daya dukung telah melampaui ambang batas")
            else:
                pdf.add_paragraph(f"STATUS: {status.upper()}")
    else:
        pdf.add_paragraph("Data daya dukung akan dilengkapi berdasarkan hasil kajian.")

    pdf.add_section("DAYA TAMPUNG LINGKUNGAN HIDUP")
    pdf.add_paragraph(
        "Daya tampung lingkungan hidup menggambarkan kemampuan lingkungan hidup untuk menyerap "
        "zat, energi, dan/atau komponen lain yang masuk atau dimasukkan ke dalamnya. Analisis "
        "daya tampung dilakukan terhadap media air, udara, dan tanah."
    )

    pdf.add_section("REKOMENDASI KEBIJAKAN")
    pdf.add_paragraph(
        "Berdasarkan hasil analisis daya dukung dan daya tampung lingkungan hidup, "
        "direkomendasikan hal-hal berikut:"
    )
    if daya_dukung_data:
        for i, item in enumerate(daya_dukung_data, 1):
            rekomendasi = item.get("rekomendasi", f"Pengelolaan {item.get('aspek', 'komponen')} secara berkelanjutan")
            pdf.add_paragraph(f"{i}. {rekomendasi}")
    pdf.add_paragraph(
        "\nRekomendasi di atas harus diintegrasikan dalam kebijakan, rencana, dan/atau program "
        "pemerintah daerah guna menjamin keberlanjutan lingkungan hidup."
    )

    pdf.output(output_path)
    return f"SUCCESS: KLHS disimpan di {output_path}"


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: amdal_engine.py <command> [args...]")
        print("Commands: ka_andal, andal, rkl_rpl, ukl_upl, klhs")
        sys.exit(1)

    command = sys.argv[1]

    try:
        if command == "ka_andal":
            # args: project_name, location, project_type, rona_json, output_path
            if len(sys.argv) < 7:
                print("ERROR: ka_andal memerlukan 5 argumen: project_name location project_type rona_json output_path")
                sys.exit(1)
            project_name = sys.argv[2]
            location = sys.argv[3]
            project_type = sys.argv[4]
            rona_data = json.loads(sys.argv[5])
            output_path = sys.argv[6]
            os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else ".", exist_ok=True)
            print(generate_ka_andal(project_name, location, project_type, rona_data, output_path))

        elif command == "andal":
            if len(sys.argv) < 6:
                print("ERROR: andal memerlukan 4 argumen: project_name location impacts_json output_path")
                sys.exit(1)
            project_name = sys.argv[2]
            location = sys.argv[3]
            impacts_data = json.loads(sys.argv[4])
            output_path = sys.argv[5]
            os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else ".", exist_ok=True)
            print(generate_andal(project_name, location, impacts_data, output_path))

        elif command == "rkl_rpl":
            if len(sys.argv) < 6:
                print("ERROR: rkl_rpl memerlukan 4 argumen: project_name location management_json output_path")
                sys.exit(1)
            project_name = sys.argv[2]
            location = sys.argv[3]
            management_data = json.loads(sys.argv[4])
            output_path = sys.argv[5]
            os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else ".", exist_ok=True)
            print(generate_rkl_rpl(project_name, location, management_data, output_path))

        elif command == "ukl_upl":
            if len(sys.argv) < 6:
                print("ERROR: ukl_upl memerlukan 4 argumen: project_name location impacts_json output_path")
                sys.exit(1)
            project_name = sys.argv[2]
            location = sys.argv[3]
            impacts_data = json.loads(sys.argv[4])
            output_path = sys.argv[5]
            os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else ".", exist_ok=True)
            print(generate_ukl_upl(project_name, location, impacts_data, output_path))

        elif command == "klhs":
            if len(sys.argv) < 5:
                print("ERROR: klhs memerlukan 3 argumen: policy_name daya_dukung_json output_path")
                sys.exit(1)
            policy_name = sys.argv[2]
            daya_dukung_data = json.loads(sys.argv[3])
            output_path = sys.argv[4]
            os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else ".", exist_ok=True)
            print(generate_klhs(policy_name, daya_dukung_data, output_path))

        else:
            print(f"ERROR: Perintah '{command}' tidak dikenal. Gunakan: ka_andal, andal, rkl_rpl, ukl_upl, klhs")
            sys.exit(1)

    except json.JSONDecodeError as e:
        print(f"ERROR: Gagal parsing JSON: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)
