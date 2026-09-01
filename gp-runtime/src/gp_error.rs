use annotate_snippets::{Level, Snippet, AnnotationKind, Renderer};

#[non_exhaustive]
#[derive(Debug)]
pub enum GpError {
    Any(String),
    Unknown(String),
}

impl GpError {
    pub fn from_stderr(stderr: String) -> Self {
        if let Some(err) = Self::checked_from_stderr(&stderr) {
            err
        } else {
            Self::Unknown(stderr)
        }
    }

    fn checked_from_stderr(stderr: &str) -> Option<Self> {
        let raw = raw_error(stderr)?;

        Some(Self::Any(raw.join("\n")))
    }

    pub fn pretty(&self, location: &str) -> String {
        let (message, snippet) = match self {
            Self::Any(snippet) => ("gp execution failed", snippet),
            Self::Unknown(snippet) => ("gp execution failed with unknown error", snippet),
        };

        let snippet = Level::ERROR.primary_title(format!("{message} at {location}")).element({
            Snippet::source(snippet).annotation(AnnotationKind::Visible.span(0..snippet.len()))
        });

        Renderer::styled().anonymized_line_numbers(true).render(&[snippet]).to_string()
    }
}

fn raw_error(err: &str) -> Option<Vec<&str>> {
    err.lines().map(|line| preceded(line)).collect()
}

fn preceded(line: &str) -> Option<&str> {
    line.trim().strip_prefix("***")
}