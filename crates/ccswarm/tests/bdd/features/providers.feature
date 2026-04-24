Feature: Pipeline execution routes to the right provider CLI

  As a user running ccswarm on different coding-agent CLIs,
  I want my flow YAML to stay portable,
  so that switching providers is a config change, not a rewrite.

  Scenario: Claude is the default provider
    When I resolve the provider for a flow that doesn't specify one
    Then the resolved provider is "claude"

  Scenario: A flow stage can pin a specific provider
    When I resolve the provider for a flow stage declaring "codex"
    Then the resolved provider is "codex"

  Scenario: The CCSWARM_PROVIDER environment variable overrides the default
    Given the environment variable CCSWARM_PROVIDER is "codex"
    When I resolve the provider for a flow that doesn't specify one
    Then the resolved provider is "codex"

  Scenario: The stage-level provider wins over the environment variable
    Given the environment variable CCSWARM_PROVIDER is "codex"
    When I resolve the provider for a flow stage declaring "claude"
    Then the resolved provider is "claude"

  Scenario Outline: Aliases parse to the expected provider kind
    When I parse provider name "<input>"
    Then the provider kind is "<kind>"

    Examples:
      | input          | kind    |
      | claude         | claude  |
      | Claude         | claude  |
      | claude-code    | claude  |
      | codex          | codex   |
      | copilot        | copilot |
      | gh-copilot     | copilot |
      | github-copilot | copilot |
