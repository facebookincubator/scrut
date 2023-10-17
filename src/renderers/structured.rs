use super::renderer::Renderer;
use crate::outcome::Outcome;

pub struct JsonRenderer(bool);

impl JsonRenderer {
    pub fn new(pretty: bool) -> Self {
        Self(pretty)
    }
}

impl Default for JsonRenderer {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Renderer for JsonRenderer {
    fn render(&self, outcomes: &[&Outcome]) -> anyhow::Result<String> {
        if self.0 {
            serde_json::to_string_pretty(outcomes)
        } else {
            serde_json::to_string(outcomes)
        }
        .map_err(anyhow::Error::new)
    }
}

pub struct YamlRenderer;

impl YamlRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for YamlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for YamlRenderer {
    fn render(&self, outcomes: &[&Outcome]) -> anyhow::Result<String> {
        serde_yaml::to_string(outcomes).map_err(anyhow::Error::new)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::JsonRenderer;
    use super::YamlRenderer;
    use crate::escaping::Escaper;
    use crate::outcome::Outcome;
    use crate::parsers::parser::ParserType;
    use crate::renderers::renderer::Renderer;
    use crate::testcase::TestCase;
    use crate::testcase::TestCaseError;

    #[test]
    fn test_json_render() {
        let renderer = JsonRenderer::default();
        let rendered = render(renderer).expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_json_render_pretty() {
        let renderer = JsonRenderer::new(true);
        let rendered = render(renderer).expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_yaml_render() {
        let renderer = YamlRenderer::new();
        let rendered = render(renderer).expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    fn render<T: Renderer>(renderer: T) -> Result<String> {
        renderer.render(&[
            &Outcome {
                output: ("the stdout", "the stderr").into(),
                testcase: TestCase {
                    title: "the title".to_string(),
                    shell_expression: "the command".to_string(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                location: None,
                result: Ok(()),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            },
            &Outcome {
                output: ("stdout 1", "stderr 1").into(),
                testcase: TestCase {
                    title: "the title 1".to_string(),
                    shell_expression: "the command 1".to_string(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                location: Some("the location 1".to_string()),
                result: Ok(()),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            },
            &Outcome {
                output: ("stdout 2", "stderr 2").into(),
                testcase: TestCase {
                    title: "the title 2".to_string(),
                    shell_expression: "the command 2".to_string(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                location: Some("the location 2".to_string()),
                result: Err(TestCaseError::InvalidExitCode {
                    actual: 123,
                    expected: 234,
                }),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            },
        ])
    }
}
