use html::root::Html;

const TRANSLATION_STYLE: &str = "max-width:calc(50% - .5em);min-width:calc(50% - .5em);padding-right:.5em;float:inline-start;overflow-wrap:break-word;hyphens:manual;text-align:end";

const HEBREW_STYLE: &str = "max-width:calc(50% - .5em);min-width:calc(50% - .5em);padding-left:.5em;float:inline-end;overflow-wrap:break-word;hyphens:manual";
const BLANK_STYLE: &str = "max-width:50%;min-width:50%;float:inline-start";
const DESCRIPTION_STYLE: &str = "max-width:100%;min-width:100%;text-align:center";
const LINE_BREAK_STYLE: &str = "clear:both";
use crate::structure::Text;
pub fn create_html(text: &Text) -> Html {
    let mut builder = Html::builder();
    let body = builder.title("weekl sicha").body(|b| {
        text.text.iter().fold(b, |b, line| {
            b.division(|d| {
                let (d, _, text) = line.commentary.iter().fold(
                    (d, 0, &line.text as &str),
                    |(d, prev_i, remaining_text), (i, commentary)| {
                        let (text, remaining_text) = remaining_text
                            .split_at(remaining_text.char_indices().nth(*i - prev_i).unwrap().0);

                        let d = d.division(|d| {
                            if let Some(translation) = &commentary.sentence_translation {
                                d.division(|d| d.style(TRANSLATION_STYLE).text(translation.clone()))
                            } else {
                                d.division(|d| d.style(BLANK_STYLE))
                            }
                            .division(|d| d.style(HEBREW_STYLE).text(text.to_string()))
                        });

                        let d = if let Some(description) = &commentary.description_paragraph {
                            d.division(|d| {
                                description
                                    .iter()
                                    .fold(d, |d, text| {
                                        d.line_break(|l| l.style(LINE_BREAK_STYLE))
                                            .text(text.clone())
                                    })
                                    .style(DESCRIPTION_STYLE)
                            })
                        } else {
                            d
                        };
                        (
                            d.line_break(|l| l.style(LINE_BREAK_STYLE)),
                            *i,
                            remaining_text,
                        )
                    },
                );

                d.division(|d| {
                    d.division(|d| d.style(BLANK_STYLE))
                        .division(|d| d.style(HEBREW_STYLE).text(text.to_string()))
                })
                .line_break(|l| l.style(LINE_BREAK_STYLE))
            })
        })
    });
    body.build()
}
