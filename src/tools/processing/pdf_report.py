#!/usr/bin/env python3
"""PDF Report Generator untuk Environmental Engineering Indonesia
Ref: PP 22/2021, template AMDAL KLHK"""

import sys
import json
import argparse
import os
from fpdf import FPDF
from datetime import datetime

class EnvReport(FPDF):
    def header(self):
        self.set_font('Helvetica', 'B', 14)
        self.cell(0, 10, self._sanitize(self.title), new_x="LMARGIN", new_y="NEXT", align='C')
        self.set_font('Helvetica', '', 9)
        self.cell(0, 5, f'Dicetak: {datetime.now().strftime("%d %B %Y, %H:%M WITA")} | ZeroClaw Environmental AI', new_x="LMARGIN", new_y="NEXT", align='C')
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
        
        # Check if the text is an image tag [IMG:path_to_image]
        if text.startswith("[IMG:") and text.endswith("]"):
            img_path = text[5:-1].strip()
            if os.path.exists(img_path):
                # Calculate width to fit page, maintaining aspect ratio
                # A4 width is 210mm, margins are 10mm each side, so max width is 190mm
                try:
                    self.image(img_path, w=170)
                    self.ln(3)
                except Exception as e:
                    self.multi_cell(0, 5, f"[Error loading image: {img_path} - {e}]")
            else:
                 self.multi_cell(0, 5, f"[Image not found: {img_path}]")
        else:
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
    try:
        pdf = EnvReport()
        pdf.alias_nb_pages()
        pdf.title = title
        pdf.add_page()

        for sec_title, sec_body in sections:
            pdf.chapter_title(sec_title)
            # Support multiple paragraphs/images in one section
            if isinstance(sec_body, list):
                 for item in sec_body:
                     pdf.chapter_body(item)
            else:
                 pdf.chapter_body(sec_body)

        pdf.output(output_path)
        return f"SUCCESS: Laporan PDF berhasil disimpan di {output_path}"
    except Exception as e:
        return f"ERROR generating PDF: {e}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--title", required=True)
    parser.add_argument("--sections", required=True, help="JSON array of [title, body] pairs. Body can be string or array of strings/images.")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    try:
        sections = json.loads(args.sections)
        print(generate_report(args.title, sections, args.output))
    except Exception as e:
        print(f"ERROR parsing inputs: {e}")
