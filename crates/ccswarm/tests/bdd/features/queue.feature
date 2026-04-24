Feature: Queue accumulates tasks for later batch execution

  As a developer who collects work throughout the day,
  I want to queue tasks and process them in one drain,
  so that I only answer OK/NG questions once at the end.

  Background:
    Given a ccswarm workspace

  Scenario: An inline task is queued
    When I queue the task "Add a login form"
    Then the queue shows 1 pending task

  Scenario: A task can be loaded from a file
    Given a task description file with content "Create tetris.html"
    When I queue the task from that file
    Then the queue shows 1 pending task
    And the most recent queued task mentions "Create tetris.html"

  Scenario: A task can be loaded from stdin
    When I queue the task from stdin with content "Ship the release"
    Then the queue shows 1 pending task
    And the most recent queued task mentions "Ship the release"

  Scenario: Clearing only removes pending tasks
    Given a queued task "A"
    And a queued task "B"
    When I clear the queue
    Then the queue shows 0 pending tasks

  Scenario: Specifying more than one input source is rejected
    When I queue a task with both --file and a positional argument
    Then queuing fails with a helpful error
