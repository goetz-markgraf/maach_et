use std::error::Error;

mod save;

pub trait Tool: 'static {
    fn get_description(&self) -> String;
    fn get_indicator(&self) -> String;
    fn execute(&self, parameter: Option<&str>, content: &str) -> Option<String>;
}

pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(save::SaveTool::new())]
}

pub fn get_tool_prompt() -> String {
    let intro = r#"
# List of tools provided

The following tools should be used to help the user in their tasks. Each tool
has a specific function and purpose.

For each tool, you will be given a description of its functionality, a usage pattern and an output example.
If you want to access to tool, you have to format you intent following the usage pattern.
This pattern is always a markdown code block. The three backticks at the beginning are followed by the
tool indicator. Behind that there is an optinal parameter.

Like so:

```tool_indicator <optional_parameter>"
content
```

If the tool has an output, it will be formatted following the given example.

There are tools that do not have an output, like writing or patching a file's content.
There are other tool that have output like reading a folder'systems or a file' s content.

You can activate any number of tools without an output but only one tool that has an output.
You will then be given this output as you next user prompt.

    "#;

    let mut prompt = intro.to_string();

    // Add descriptions from all available tools
    for tool in get_all_tools() {
        prompt.push_str(&tool.get_description());
        prompt.push_str("\n");
    }

    prompt
}
