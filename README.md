# randy

## What is this?

A game with AI features to guess numbers. You pick a range, you guess a number and an AI answers
back in a cowbow-like manner.

That's it.

Read the [instructions] to set up the program, and the [features] to learn about further
possibilities with the program.

## Instructions <instructions>

Everything's in the `help` command page, though it is also detailed here.

- You're going to need an API from OpenRouter to use the application. Just go to your API key
  [settings] and generate a new one.
- The program will read either the environment variable or the command-line argument with the API
  key, prioritising the second if both are present.
  - To specifiy the environment variable, set the following in either one of your shell profile or
    right before the command to run the program:
    ```
    OPENROUTER_API=<YOUR_API>
    ```
  - To specify the argument to the program, run it with the `api-key` option:
    ```
    randy --api-key <YOUR_API>
    ```

## Features <features>

- The program can pick which model to use for the AI answer. The model must be specified through
  either one of the corresponding environment variable or command line argument.
  The default, if no option is specified, is DeepSeek's V3.
- The model name must follow OpenRouter naming conventions, i.e. you must check the model's name in
  the OpenRouter model at their [models]' page. The name to look for is the smaller one below the
  public-facing name.
- To specify the environment variable, set the following variable in either one of your shell
  profile or inline right before the program's name.
  ```
  OPENROUTER_MODEL=<MODEL_NAME>
  ```
- To specify the command-line argument, pass the `model` option to the program:
  ```
  randy --model <MODEL_NAME>
  ```

## Install

### Crates.io

```
cargo install randyrand
```

### Source

```
git clone https://github.com/dybuc/randy
cd randy
cargo install --no-track --locked --path .
```

[settings]: https://openrouter.ai/settings/keys
[models]: https://openrouter.ai/models
