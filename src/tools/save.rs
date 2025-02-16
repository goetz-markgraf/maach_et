use std::error::Error;

use super::Tool;

pub struct SaveTool;

impl SaveTool {
    pub fn new() -> Self {
        SaveTool {}
    }
}

impl Tool for SaveTool {
    fn get_description(&self) -> String {
        let prompt = r#"
## Save Tool

### Purpose:
Create or overwrite a file with the given content.
Whenever you want to create a overwrite a file, always use this tool. Do not just
print the code to the user.

### Usage Pattern:

The path can be relative to the current directory, or absolute.
If the current directory changes, the path will be relative to the new directory.

To write to a file, use a code block with the language tag: `save <path>`

Example:

```save hello_world.rs
fn main() {
    println!("Hello, world!");
}
```

### Output:

no output

        "#;

        prompt.to_string()
    }

    fn get_indicator(&self) -> String {
        "save".to_string()
    }

    fn execute(
        &self,
        parameter: Option<&str>,
        content: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let path = parameter.ok_or_else(|| "No file path provided".to_string())?;
        std::fs::write(path, content)?;
        Ok(None)
    }
}
