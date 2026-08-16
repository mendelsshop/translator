use pdf_oxide::api::{Pdf, PdfBuilder};

use crate::structure::Text;

pub fn create_pdf(text: &Text) -> Pdf {
    let mut pdf = PdfBuilder::new().from_text("Sicha Weekly").unwrap();
    pdf.set_title("foobar");
    for _line in &text.text {}
    pdf
}
