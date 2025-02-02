use crate::tools::get_all_tools;

#[derive(Debug, PartialEq)]
pub struct ToolInput {
    pub name: String,
    pub parameter: Option<String>,
    pub content: String,
}

pub fn check_for_tools(llm_output: String) -> Vec<ToolInput> {
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

pub fn run_tools(llm_output: &str) -> Option<String> {
    todo!("not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_tool_blocks() {
        let input = "This is a normal message without any tool blocks.".to_string();
        assert_eq!(check_for_tools(input), Vec::new());
    }

    #[test]
    fn test_single_block_with_parameter() {
        let input = r#"Here's a tool block:
```save test.txt
Hello, world!
```"#
            .to_string();

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
```"#
            .to_string();

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
```"#
            .to_string();

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
}
