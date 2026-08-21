use html::root::Html;
use itertools::Itertools;

use crate::structure::Text;
pub fn create_html(text: &Text) -> Html {
    let mut builder = Html::builder();
    let body = builder.title("weekl sicha").body(|b| {
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

                        let d = d.division(|d|if let Some(translation) = &commentary.sentence_translation {
                            d.division(|d| {
                                d.style("max-width:50%; min-width:50%; float: inline-start; overflow-wrap: break-word; hyphens: manual; text-align: end;").text(translation.clone())
                            })
                        } else {
                            d.division(|d| d.style("max-width:50%; min-width:50%; float: inline-start"))
                        }.division(|d| {
                                d.style("max-width:50%; min-width:50%; float: inline-end; overflow-wrap: break-word; hyphens: manual; ").text(text.to_string())
                            })
                            );
                        (
                            d.line_break(|l| l.style("clear:both")),
                            *i,
                            remaining_text,
                        )
                    },
                );

                            d.division(|d|d.division(|d| d.style("max-width:50%; min-width:50%; float: inline-start"))
                            .division(|d|d.style("max-width:50%; min-width:50%; float: inline-end; overflow-wrap: break-word; hyphens: manual; ").text(text.to_string()))).line_break(|l| l.style("clear:both"))
            })
        })
    });
    body.build()
}
