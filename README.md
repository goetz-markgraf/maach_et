# maach_et
(German Cologne Dialect for: "do it!")

A LLM agent tool for the CLI written in Rust

Currently WIP

## Features

- LLM agonstic Client. Currently implemented: local Ollama and ChatGPT
- Chat-Loop for interactive conversations
- System prompt
- Tool architecture for extensibility and integration of various functionalities

Implemented Tools
- save tool (with just debugging functionality, no actual write to disk yet)

## Problems

currently my local Ollama model (`deepseek-r1`) is not using the tools provided.
Specially when creating a file, it always uses just an code block and does not
use the implemented save tool