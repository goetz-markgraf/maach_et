use crate::tools::{get_all_tools, Tool};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct ToolInput {
    pub name: String,
    pub parameter: Option<String>,
    pub content: String,
}

pub fn run_tools(llm_output: &str) -> Option<String> {
    let tool_candidates = check_for_tools(llm_output);
    let all_tools = get_all_tools();

    execute_tools(tool_candidates, all_tools)
}

fn execute_tools(tool_candidates: Vec<ToolInput>, all_tools: Vec<Box<dyn Tool>>) -> Option<String> {
    for tool_input in tool_candidates {
        for tool in &all_tools {
            if tool.get_indicator() == tool_input.name {
                if let Some(result) =
                    tool.execute(tool_input.parameter.as_deref(), &tool_input.content)
                {
                    return Some(result);
                }
            }
        }
    }
    None
}

fn check_for_tools(llm_output: &str) -> Vec<ToolInput> {
    let mut tool_inputs = Vec::new();
    let mut current_pos = 0;

    while let Some(start) = llm_output[current_pos..].find("```") {
        let start = current_pos + start + 3; // Skip the backticks

        if let Some(newline) = llm_output[start..].find('\n') {
            let tool_line = llm_output[start..start + newline].trim();
            let mut parts = tool_line.split_whitespace();

            if let Some(name) = parts.next() {
                let parameter = parts.collect::<Vec<_>>().join(" ");
                let parameter = if parameter.is_empty() {
                    None
                } else {
                    Some(parameter)
                };

                let content_start = start + newline + 1;
                if let Some(end) = llm_output[content_start..].find("```") {
                    let content = llm_output[content_start..content_start + end]
                        .trim()
                        .to_string();
                    tool_inputs.push(ToolInput {
                        name: name.to_string(),
                        parameter,
                        content,
                    });
                    current_pos = content_start + end + 3;
                    continue;
                }
            }
        }
        current_pos = start;
    }

    tool_inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockTool {
        indicator: String,
        result: Option<String>,
        was_called: Rc<RefCell<bool>>,
    }

    impl MockTool {
        fn new(indicator: &str, result: Option<String>) -> Self {
            Self {
                indicator: indicator.to_string(),
                result,
                was_called: Rc::new(RefCell::new(false)),
            }
        }

        fn was_called(&self) -> bool {
            *self.was_called.borrow()
        }
    }

    impl Tool for MockTool {
        fn get_description(&self) -> String {
            "Mock tool".to_string()
        }

        fn get_indicator(&self) -> String {
            self.indicator.clone()
        }

        fn execute(&self, _parameter: Option<&str>, _content: &str) -> Option<String> {
            *self.was_called.borrow_mut() = true;
            self.result.clone()
        }
    }

    #[test]
    fn test_no_tool_blocks() {
        let input = "This is a normal message without any tool blocks.";
        assert_eq!(check_for_tools(input), Vec::new());
    }

    #[test]
    fn test_single_block_with_parameter() {
        let input = r#"Here's a tool block:
```save test.txt
Hello, world!
```"#;

        assert_eq!(
            check_for_tools(input),
            vec![ToolInput {
                name: "save".to_string(),
                parameter: Some("test.txt".to_string()),
                content: "Hello, world!".to_string(),
            }]
        );
    }

    #[test]
    fn test_single_block_without_parameter() {
        let input = r#"Here's a tool block:
```list
directory contents
```"#;

        assert_eq!(
            check_for_tools(input),
            vec![ToolInput {
                name: "list".to_string(),
                parameter: None,
                content: "directory contents".to_string(),
            }]
        );
    }

    #[test]
    fn test_multiple_blocks() {
        let input = r#"Here are multiple blocks:
```save test.txt
Hello, world!
```
Some text in between
```list
directory contents
```"#;

        assert_eq!(
            check_for_tools(input),
            vec![
                ToolInput {
                    name: "save".to_string(),
                    parameter: Some("test.txt".to_string()),
                    content: "Hello, world!".to_string(),
                },
                ToolInput {
                    name: "list".to_string(),
                    parameter: None,
                    content: "directory contents".to_string(),
                }
            ]
        );
    }

    #[test]
    fn test_execute_tools_last_returns_some() {
        let tool1 = MockTool::new("tool1", None);
        let tool2 = MockTool::new("tool2", None);
        let tool3 = MockTool::new("tool3", Some("success".to_string()));

        let tool_candidates = vec![
            ToolInput {
                name: "tool1".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
            ToolInput {
                name: "tool2".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
            ToolInput {
                name: "tool3".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
        ];

        let all_tools: Vec<Box<dyn Tool>> = vec![
            Box::new(tool1.clone()),
            Box::new(tool2.clone()),
            Box::new(tool3.clone()),
        ];

        let result = execute_tools(tool_candidates, all_tools);

        assert_eq!(result, Some("success".to_string()));
        assert!(tool1.was_called());
        assert!(tool2.was_called());
        assert!(tool3.was_called());
    }

    #[test]
    fn test_execute_tools_all_return_none() {
        let tool1 = MockTool::new("tool1", None);
        let tool2 = MockTool::new("tool2", None);
        let tool3 = MockTool::new("tool3", None);

        let tool_candidates = vec![
            ToolInput {
                name: "tool1".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
            ToolInput {
                name: "tool2".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
            ToolInput {
                name: "tool3".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
        ];

        let all_tools: Vec<Box<dyn Tool>> = vec![
            Box::new(tool1.clone()),
            Box::new(tool2.clone()),
            Box::new(tool3.clone()),
        ];

        let result = execute_tools(tool_candidates, all_tools);

        assert_eq!(result, None);
        assert!(tool1.was_called());
        assert!(tool2.was_called());
        assert!(tool3.was_called());
    }

    #[test]
    fn test_execute_tools_first_returns_some() {
        let tool1 = MockTool::new("tool1", Some("early success".to_string()));
        let tool2 = MockTool::new("tool2", Some("should not see this".to_string()));
        let tool3 = MockTool::new("tool3", Some("should not see this either".to_string()));

        let tool_candidates = vec![
            ToolInput {
                name: "tool1".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
            ToolInput {
                name: "tool2".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
            ToolInput {
                name: "tool3".to_string(),
                parameter: None,
                content: "test".to_string(),
            },
        ];

        let all_tools: Vec<Box<dyn Tool>> = vec![
            Box::new(tool1.clone()),
            Box::new(tool2.clone()),
            Box::new(tool3.clone()),
        ];

        let result = execute_tools(tool_candidates, all_tools);

        assert_eq!(result, Some("early success".to_string()));
        assert!(tool1.was_called());
        assert!(!tool2.was_called());
        assert!(!tool3.was_called());
    }
}
