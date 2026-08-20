use html::{root::Html, text_content::Division};
use itertools::Itertools;

use crate::structure::Text;
pub fn create_html(text: &Text) -> Html {
    let mut builder = Html::builder();
    let body = builder.title("weekl sicha").body(|mut b| {
        text.text.iter().fold(b, |b, line| {
            b.division(|d| {
                let (d, _, text) = line.commentary.iter().fold(
                    (d, 0, &line.text as &str),
                    |(d, prev_i, remaining_text), (i, commentary)| {
                        log::info!(
                            "{i} {prev_i} {remaining_text} {} {}",
                            remaining_text.chars().collect_vec().len(),
                            *i - prev_i
                        );
                        let (text, remaining_text) = remaining_text
                            .split_at(remaining_text.char_indices().nth(*i - prev_i).unwrap().0);
                        (
                            d.text(text.to_string()).line_break(|l| l),
                            *i,
                            remaining_text,
                        )
                    },
                );
                d.text(text.to_string()).line_break(|l| l)
            })
        })
    });
    body.build()
}
