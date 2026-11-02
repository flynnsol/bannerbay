//! `*emphasis*` markers in header text.
//!
//! `Ship your *launch banner* in seconds` marks "launch banner" for the highlight colour.
//! `\*` is a literal asterisk, and an asterisk with no partner stays literal. Everything here
//! is pure, so it is tested natively and the renderer only has to draw the spans.

/// A run of text that is either plain or emphasised.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Span {
    pub text: String,
    pub emphasis: bool,
}

impl Span {
    fn new(text: impl Into<String>, emphasis: bool) -> Self {
        Self { text: text.into(), emphasis }
    }
}

/// True if an unescaped `*` closes an emphasis opened at `open`, with at least one char between.
fn has_closer(chars: &[char], open: usize) -> bool {
    let mut i = open + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' if chars.get(i + 1) == Some(&'*') => i += 2,
            '*' => return i > open + 1,
            _ => i += 1,
        }
    }
    false
}

/// Splits `input` into plain and emphasised spans, dropping the markers.
pub fn parse(input: &str) -> Vec<Span> {
    let chars: Vec<char> = input.chars().collect();
    let mut spans: Vec<Span> = Vec::new();
    let (mut cur, mut emphasis, mut i) = (String::new(), false, 0);
    let flush = |spans: &mut Vec<Span>, cur: &mut String, emphasis: bool| {
        if !cur.is_empty() {
            spans.push(Span::new(std::mem::take(cur), emphasis));
        }
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && chars.get(i + 1) == Some(&'*') {
            cur.push('*');
            i += 2;
        } else if c == '*' && emphasis {
            flush(&mut spans, &mut cur, true);
            emphasis = false;
            i += 1;
        } else if c == '*' && has_closer(&chars, i) {
            flush(&mut spans, &mut cur, false);
            emphasis = true;
            i += 1;
        } else {
            cur.push(c);
            i += 1;
        }
    }
    // An opener always has a closer, so `emphasis` is false here; flush what is left.
    flush(&mut spans, &mut cur, emphasis);
    spans
}

/// One unbreakable word, which may mix plain and emphasised pieces (`*launch*,`).
type Word = Vec<Span>;

fn word_len(w: &Word) -> usize {
    w.iter().map(|s| s.text.chars().count()).sum()
}

/// Appends `span` to `line`, merging with the previous span when the emphasis matches.
fn push_merged(line: &mut Vec<Span>, span: Span) {
    match line.last_mut() {
        Some(last) if last.emphasis == span.emphasis => last.text.push_str(&span.text),
        _ => line.push(span),
    }
}

/// Greedy word wrap over spans. Matches `render::wrap` on plain text: explicit newlines break
/// lines, blank lines are dropped, a word longer than the width stays whole, and the width is
/// never below 8 characters. A space between two emphasised words is emphasised too, so a
/// highlighted phrase stays one span.
pub fn wrap(spans: &[Span], max_chars: usize) -> Vec<Vec<Span>> {
    let max_chars = max_chars.max(8);

    // 1. Split into words and hard breaks (`None`).
    let mut items: Vec<Option<Word>> = Vec::new();
    let mut word: Word = Vec::new();
    for span in spans {
        let mut piece = String::new();
        for c in span.text.chars() {
            if c.is_whitespace() {
                if !piece.is_empty() {
                    word.push(Span::new(std::mem::take(&mut piece), span.emphasis));
                }
                if !word.is_empty() {
                    items.push(Some(std::mem::take(&mut word)));
                }
                if c == '\n' {
                    items.push(None);
                }
            } else {
                piece.push(c);
            }
        }
        if !piece.is_empty() {
            word.push(Span::new(piece, span.emphasis));
        }
    }
    if !word.is_empty() {
        items.push(Some(word));
    }

    // 2. Pack words into lines.
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut line: Vec<Span> = Vec::new();
    let mut line_len = 0;
    let mut last_emphasis = false;
    let finish = |lines: &mut Vec<Vec<Span>>, line: &mut Vec<Span>, len: &mut usize| {
        if !line.is_empty() {
            lines.push(std::mem::take(line));
        }
        *len = 0;
    };
    for item in items {
        let Some(w) = item else {
            finish(&mut lines, &mut line, &mut line_len);
            continue;
        };
        let len = word_len(&w);
        if !line.is_empty() && line_len + 1 + len > max_chars {
            finish(&mut lines, &mut line, &mut line_len);
        }
        if !line.is_empty() {
            // The joining space is emphasised only between two emphasised words.
            let first_emphasis = w.first().is_some_and(|s| s.emphasis);
            push_merged(&mut line, Span::new(" ", last_emphasis && first_emphasis));
            line_len += 1;
        }
        last_emphasis = w.last().is_some_and(|s| s.emphasis);
        line_len += len;
        for span in w {
            push_merged(&mut line, span);
        }
    }
    finish(&mut lines, &mut line, &mut line_len);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(spans: &[Span]) -> String {
        spans.iter().map(|s| s.text.as_str()).collect()
    }

    fn sp(text: &str, emphasis: bool) -> Span {
        Span::new(text, emphasis)
    }

    #[test]
    fn parses_a_marked_phrase() {
        assert_eq!(
            parse("Ship your *launch banner* in seconds"),
            vec![sp("Ship your ", false), sp("launch banner", true), sp(" in seconds", false)]
        );
    }

    #[test]
    fn parses_several_marked_phrases_and_edges() {
        assert_eq!(parse("*a* b *c*"), vec![sp("a", true), sp(" b ", false), sp("c", true)]);
        assert_eq!(parse("*all*"), vec![sp("all", true)]);
        assert_eq!(parse("no markers"), vec![sp("no markers", false)]);
        assert!(parse("").is_empty());
    }

    #[test]
    fn unbalanced_and_empty_markers_stay_literal() {
        assert_eq!(plain(&parse("5 * 3")), "5 * 3");
        assert_eq!(parse("5 * 3"), vec![sp("5 * 3", false)]);
        assert_eq!(plain(&parse("a ** b")), "a ** b");
        assert_eq!(parse("trailing *"), vec![sp("trailing *", false)]);
    }

    #[test]
    fn escaped_asterisk_is_literal_and_does_not_close() {
        assert_eq!(parse(r"2 \* 3"), vec![sp("2 * 3", false)]);
        assert_eq!(parse(r"*a \* b*"), vec![sp("a * b", true)]);
        assert_eq!(parse(r"\*not emphasis\*"), vec![sp("*not emphasis*", false)]);
    }

    #[test]
    fn markers_never_appear_in_the_text() {
        for input in ["*a* b", "x *y z* w", "*one* *two* *three*"] {
            assert!(!plain(&parse(input)).contains('*'), "{input}");
        }
    }

    #[test]
    fn emphasis_can_span_a_line_break() {
        let lines = wrap(&parse("*a\nb* c"), 40);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], vec![sp("a", true)]);
        assert_eq!(lines[1], vec![sp("b", true), sp(" c", false)]);
    }

    #[test]
    fn wrap_matches_plain_wrap_without_markers() {
        for (text, width) in [
            ("aaa bbb ccc", 8),
            ("one\ntwo", 20),
            ("   ", 20),
            ("supercalifragilistic", 8),
            ("the quick brown fox jumps over the lazy dog", 12),
            ("a\n\nb   c", 20),
        ] {
            let got: Vec<String> = wrap(&parse(text), width).iter().map(|l| plain(l)).collect();
            assert_eq!(got, crate::render::wrap(text, width), "{text:?} at {width}");
        }
    }

    #[test]
    fn wrap_keeps_a_highlighted_phrase_in_one_span() {
        let lines = wrap(&parse("Ship your *launch banner* in seconds"), 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], vec![sp("Ship your ", false), sp("launch banner", true), sp(" in seconds", false)]);
    }

    #[test]
    fn wrap_breaks_inside_a_highlighted_phrase() {
        let lines = wrap(&parse("aa *bb cc dd* ee"), 8);
        assert_eq!(lines.iter().map(|l| plain(l)).collect::<Vec<_>>(), vec!["aa bb cc", "dd ee"]);
        assert_eq!(lines[0], vec![sp("aa ", false), sp("bb cc", true)]);
        assert_eq!(lines[1], vec![sp("dd", true), sp(" ee", false)]);
    }

    #[test]
    fn punctuation_next_to_a_marker_stays_attached() {
        let lines = wrap(&parse("go *fast*, now"), 80);
        assert_eq!(lines[0], vec![sp("go ", false), sp("fast", true), sp(", now", false)]);
        // The comma is part of the same unbreakable word as "fast".
        let narrow: Vec<String> = wrap(&parse("go *fast*, now"), 8).iter().map(|l| plain(l)).collect();
        assert_eq!(narrow, vec!["go fast,", "now"]);
    }

    #[test]
    fn markers_take_no_width() {
        // "abcd efgh" is 9 chars; with markers the visible text is the same, so it wraps the same.
        assert_eq!(wrap(&parse("*abcd* *efgh*"), 9).len(), 1);
        assert_eq!(wrap(&parse("abcd efgh"), 9).len(), 1);
        assert_eq!(wrap(&parse("*abcd* *efgh*"), 8).len(), 2);
    }
}
