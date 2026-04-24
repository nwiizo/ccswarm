Feature: ccswarm suggests an appropriate flow for a task

  As a user facing a choice between several builtin flows,
  I want ccswarm to propose a reasonable default for my task,
  so that I don't have to memorize when to use each one.

  Scenario Outline: The suggested flow matches the task's character
    When I ask ccswarm to suggest a flow for "<task>"
    Then the suggested flow is "<flow>"

    Examples:
      | task                                                                      | flow       |
      | Rename foo to bar                                                         | quick      |
      | Fix the regression breaking login on Safari when cookies are disabled     | review-fix |
      | Investigate why cache latency grew 40% last week                          | research   |
      | Add a full-stack dashboard with frontend and backend endpoints            | team       |
      | Design and roll out an audit-trail feature for tenant configuration edits spanning the dashboard, admin API, and nightly export job | default    |
