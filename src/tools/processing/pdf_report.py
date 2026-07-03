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
        self.cell(0, 10, self._sanitize(self.title), new_x="LMARGIN", new_y="NEXT")
        self.set_font('Helvetica', '', 9)
        self.cell(0, 5, f'Dicetak: {datetime.now().strftime("%d %B %Y, %H:%M WITA")} | ZeroClaw Environmental AI', new_x="LMARGIN", new_y="NEXT")
        self.line(10, self.get_y()+2, 200, self.get_y()+2)
        self.ln(5)

    def footer(self):
        self.set_y(-15)
        self.set_font('Helvetica', 'I', 8)
        self.cell(0, 10, f'Halaman {self.page_no()}/{{nb}} | Domain: Indonesia | Physics-Informed', align='C')

    def chapter_title(self, title):
        self.set_font('Helvetica', 'B', 12)
        self.set_fill_color(41, 128, 185)
        self.set_text_color(255, 255, 255)
        self.cell(0, 8, f'  {self._sanitize(title)}', new_x="LMARGIN", new_y="NEXT", fill=True)
        self.set_text_color(0, 0, 0)
        self.ln(3)

    def chapter_body(self, text):
        self.set_font('Helvetica', '', 10)
        self.multi_cell(0, 5, self._sanitize(text))
        self.ln(3)

    @staticmethod
    def _sanitize(text):
        """Replace Unicode chars that Helvetica can't render"""
        replacements = {
            '\u2192': '->', '\u2190': '<-', '\u2194': '<->',
            '\u2713': '[v]', '\u2717': '[x]', '\u2022': '*',
            '\u2264': '<=', '\u2265': '>=', '\u2260': '!=',
            '\u00b2': '2', '\u00b3': '3', '\u00b0': 'deg',
            '\u03bc': 'u', '\u2103': 'C', '\u2109': 'F',
            '\u00d7': 'x', '\u00f7': '/',
            '\u2019': "'", '\u201c': '"', '\u201d': '"',
            '\u2014': '--', '\u2013': '-',
        }
        for k, v in replacements.items():
            text = text.replace(k, v)
        # Fallback: replace any remaining non-latin1 chars
        return text.encode('latin-1', errors='replace').decode('latin-1')

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
