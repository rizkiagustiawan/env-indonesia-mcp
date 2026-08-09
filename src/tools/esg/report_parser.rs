// ESG Report Parser — STUB (not implemented)
// NOTE: This tool is a placeholder. It does NOT parse PDFs or assess ESG compliance.
// To implement: use lopdf/PyPDF2 for text extraction + LLM for GRI/TCFD assessment.

pub async fn parse_esg_report(pdf_path: &str) -> String {
    format!(
        "=== ESG Report Parser — NOT IMPLEMENTED ===\n\nFile: {}\n\nSTATUS: This tool is a STUB.\nIt does NOT parse PDFs or assess ESG compliance.\n\nTo use: manually extract text from the PDF and feed indicators to the LLM\nfor GRI/TCFD compliance assessment.\n\nAlternatively, implement with lopdf (Rust) or PyPDF2 (Python) for extraction.",
        pdf_path
    )
}
