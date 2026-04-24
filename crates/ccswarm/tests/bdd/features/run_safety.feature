Feature: Run IDs cannot escape the runs directory

  As an operator,
  I want malformed run IDs rejected before any file read,
  so that no tool-command can be tricked into returning arbitrary files.

  Background:
    Given a ccswarm workspace with a runs directory

  Scenario Outline: Suspicious run IDs are refused
    When I ask for details of run "<run-id>"
    Then the request is rejected with "run ID"

    Examples:
      | run-id                |
      | ../../etc/passwd      |
      | ..                    |
      | a/b                   |
      | a\b                   |
      | foo bar               |
      | 'DROP TABLE runs'     |

  Scenario: A well-formed UUID-style run ID is accepted for validation
    When I ask for details of run "97fc1ccd-ea1b-4119-9a3c-2c1a550d71ee"
    Then the run ID passes validation
