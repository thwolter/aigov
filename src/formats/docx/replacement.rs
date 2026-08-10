use crate::formats::docx;
use crate::office::document::{CaseMatching, ReplaceOptions};
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};

const WORD_PARAGRAPH: &[u8] = b"w:p";
const WORD_TEXT: &[u8] = b"w:t";

/// Replaces text in Word paragraphs while preserving their surrounding XML events.
pub fn replace_document_text(
    xml: &[u8],
    search: &str,
    replacement: &str,
    options: ReplaceOptions,
) -> crate::Result<(usize, Vec<u8>)> {
    if search.is_empty() {
        return Ok((0, xml.to_vec()));
    }

    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Vec::new());
    let mut buffer = Vec::new();
    let mut paragraph_events: Option<Vec<Event<'static>>> = None;
    let mut count = 0;

    loop {
        let event = reader.read_event_into(&mut buffer)?.into_owned();

        match event {
            Event::Eof => break,
            Event::Start(start) if start.name().as_ref() == WORD_PARAGRAPH => {
                if let Some(events) = paragraph_events.as_mut() {
                    events.push(Event::Start(start));
                } else {
                    paragraph_events = Some(vec![Event::Start(start)]);
                }
            }
            Event::End(end)
                if paragraph_events.is_some() && end.name().as_ref() == WORD_PARAGRAPH =>
            {
                paragraph_events
                    .as_mut()
                    .expect("paragraph exists")
                    .push(Event::End(end));

                let (replacements, events) = replace_paragraph_text(
                    paragraph_events.take().expect("paragraph exists"),
                    search,
                    replacement,
                    options.case_matching,
                )?;
                count += replacements;

                for event in events {
                    writer.write_event(event)?;
                }
            }
            event => {
                if let Some(events) = paragraph_events.as_mut() {
                    events.push(event);
                } else {
                    writer.write_event(event)?;
                }
            }
        }

        buffer.clear();
    }

    if paragraph_events.is_some() {
        return Err(docx::invalid_document("unterminated Word paragraph"));
    }

    Ok((count, writer.into_inner()))
}

/// Replaces matches spanning one or more text nodes in a single paragraph.
fn replace_paragraph_text(
    mut events: Vec<Event<'static>>,
    search: &str,
    replacement: &str,
    case_matching: CaseMatching,
) -> crate::Result<(usize, Vec<Event<'static>>)> {
    let mut in_text = false;
    let mut event_indices = Vec::new();
    let mut texts = Vec::new();

    for (index, event) in events.iter().enumerate() {
        match event {
            Event::Start(start) if start.name().as_ref() == WORD_TEXT => in_text = true,
            Event::End(end) if end.name().as_ref() == WORD_TEXT => in_text = false,
            Event::Text(text) if in_text => {
                event_indices.push(index);
                texts.push(decode_text(text)?);
            }
            _ => {}
        }
    }

    let text = texts.concat();
    let ranges = match_ranges(&text, search, case_matching);
    apply_replacements(&mut texts, &ranges, replacement);

    for (event_index, text) in event_indices.into_iter().zip(texts) {
        events[event_index] = Event::Text(BytesText::new(&text).into_owned());
    }

    Ok((ranges.len(), events))
}

/// Decodes an XML text event into its unescaped Word text value.
fn decode_text(text: &BytesText<'_>) -> crate::Result<String> {
    let text = text
        .xml10_content()
        .map_err(|error| docx::invalid_document(format!("invalid Word text: {error}")))?;
    quick_xml::escape::unescape(&text)
        .map(|text| text.into_owned())
        .map_err(|error| docx::invalid_document(format!("invalid Word text: {error}")))
}

/// Finds byte ranges for matches according to the selected case policy.
fn match_ranges(text: &str, search: &str, case_matching: CaseMatching) -> Vec<(usize, usize)> {
    match case_matching {
        CaseMatching::Sensitive => text
            .match_indices(search)
            .map(|(start, matched)| (start, start + matched.len()))
            .collect(),
        CaseMatching::UnicodeInsensitive => case_insensitive_ranges(text, search),
    }
}

/// Finds case-insensitive matches while mapping ranges back to the original UTF-8 text.
fn case_insensitive_ranges(text: &str, search: &str) -> Vec<(usize, usize)> {
    let folded_search = search.to_lowercase();
    let mut folded_text = String::new();
    let mut boundaries = vec![(0, 0)];

    for (start, character) in text.char_indices() {
        folded_text.extend(character.to_lowercase());
        boundaries.push((folded_text.len(), start + character.len_utf8()));
    }

    folded_text
        .match_indices(&folded_search)
        .filter_map(|(start, matched)| {
            let end = start + matched.len();
            let start_index = boundaries
                .binary_search_by_key(&start, |entry| entry.0)
                .ok()?;
            let end_index = boundaries
                .binary_search_by_key(&end, |entry| entry.0)
                .ok()?;
            Some((boundaries[start_index].1, boundaries[end_index].1))
        })
        .collect()
}

/// Applies replacement ranges across the affected text nodes in reverse order.
fn apply_replacements(texts: &mut [String], ranges: &[(usize, usize)], replacement: &str) {
    let spans = text_spans(texts);

    for &(start, end) in ranges.iter().rev() {
        let first = spans
            .iter()
            .position(|span| span.0 <= start && start < span.1)
            .expect("match starts in a text node");
        let last = spans
            .iter()
            .position(|span| span.0 < end && end <= span.1)
            .expect("match ends in a text node");
        let start_offset = start - spans[first].0;
        let end_offset = end - spans[last].0;

        if first == last {
            texts[first].replace_range(start_offset..end_offset, replacement);
        } else {
            texts[first].replace_range(start_offset.., replacement);
            for text in &mut texts[first + 1..last] {
                text.clear();
            }
            texts[last].replace_range(..end_offset, "");
        }
    }
}

/// Returns each text node's byte range in the concatenated paragraph text.
fn text_spans(texts: &[String]) -> Vec<(usize, usize)> {
    let mut offset = 0;

    texts
        .iter()
        .map(|text| {
            let start = offset;
            offset += text.len();
            (start, offset)
        })
        .collect()
}
