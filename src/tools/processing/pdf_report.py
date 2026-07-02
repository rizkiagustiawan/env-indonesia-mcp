#!/usr/bin/env python3
"""PDF Report Generator untuk Environmental Engineering Indonesia
Ref: PP 22/2021, template AMDAL KLHK"""

import sys
import json
import argparse
from fpdf import FPDF
from datetime import datetime

class EnvReport(FPDF):
    def header(self):
        self.set_font('Helvetica', 'B', 14)
        self.cell(0, 10, self.title, 0, 1, 'C')
        self.set_font('Helvetica', '', 9)
        self.cell(0, 5, f'Dicetak: {datetime.now().strftime("%d %B %Y, %H:%M WITA")} | ZeroClaw Environmental AI', 0, 1, 'C')
        self.line(10, self.get_y()+2, 200, self.get_y()+2)
        self.ln(5)

    def footer(self):
        self.set_y(-15)
        self.set_font('Helvetica', 'I', 8)
        self.cell(0, 10, f'Halaman {self.page_no()}/{{nb}} | Domain: Indonesia | Physics-Informed', 0, 0, 'C')

    def chapter_title(self, title):
        self.set_font('Helvetica', 'B', 12)
        self.set_fill_color(41, 128, 185)
        self.set_text_color(255, 255, 255)
        self.cell(0, 8, f'  {title}', 0, 1, 'L', True)
        self.set_text_color(0, 0, 0)
        self.ln(3)

    def chapter_body(self, text):
        self.set_font('Helvetica', '', 10)
        self.multi_cell(0, 5, text)
        self.ln(3)

def generate_report(title, sections, output_path):
    pdf = EnvReport()
    pdf.alias_nb_pages()
    pdf.title = title
    pdf.add_page()

    for sec_title, sec_body in sections:
        pdf.chapter_title(sec_title)
        pdf.chapter_body(sec_body)

    pdf.output(output_path)
    return f"SUCCESS: Laporan PDF berhasil disimpan di {output_path}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--title", required=True)
    parser.add_argument("--sections", required=True, help="JSON array of [title, body] pairs")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    sections = json.loads(args.sections)
    print(generate_report(args.title, sections, args.output))
