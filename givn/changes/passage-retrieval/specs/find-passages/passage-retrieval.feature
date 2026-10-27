@givn.delta @passage-retrieval

Feature: Passage retrieval

  The agent submits a claim or phrase to the indexed library and receives
  candidate passages with citations, or an explicit no-evidence result, or an
  unavailability report.

  @givn.added @e2e @wip
  Scenario: Agent retrieves cited passages for a claim
    Given the library holds at least one indexed edition
    And an agent connected through the passage-retrieval interface
    When the agent submits the claim "Vertrauen in Organisationen" in German with scope genre=fachbuch, requesting 5 passages
    Then the agent receives at most 5 candidate passages
    And every returned passage carries a citation to an indexed edition, a score and the expansion used
    And the passages are ordered by score, then work id, then passage index

  @givn.added @e2e @wip
  Scenario: Agent widens context around a cited passage
    Given the library holds an indexed edition with a cited passage
    And an agent connected through the passage-retrieval interface
    When the agent requests wider context around the cited passage
    Then the agent receives the surrounding text with exact bounds
    And the cited passage appears verbatim inside the returned text

  @givn.added @wip
  Scenario: Scope restricts passages to the named genre
    Given the claim appears in indexed works of the genres fachbuch and sachbuch
    When the agent submits that claim with scope genre=fachbuch
    Then every returned citation belongs to an indexed work of the genre fachbuch
    And no returned citation belongs to a work outside the genre fachbuch

  @givn.added @wip
  Scenario: A scope naming an unknown work yields no evidence
    Given an indexed library
    When the agent submits a claim with a scope naming the work "unbekanntes-werk"
    Then the agent receives no evidence with reason unknown work
    And no passage is returned

  @givn.added @wip
  Scenario: No passage above the threshold yields no evidence
    Given an indexed library that holds no passage for the claim
    When the agent submits the claim
    Then the agent receives no evidence with reason no evidence
    And the result is neither a failure nor an empty success

  @givn.added @wip
  Scenario: An unavailable or stale index yields an unavailability report
    Given an index that is older than the freshness bound
    When the agent submits a claim
    Then the agent receives an unavailability report
    And the report is distinguishable from no evidence

  @givn.added @wip
  Scenario: Fewer passages than requested are returned short
    Given an indexed library where four passages reach the threshold
    When the agent requests 20 passages
    Then the agent receives 4 candidate passages
    And the result is marked short
    And no passage is padded to reach the requested number

  @givn.added @wip
  Scenario: The result is capped at twenty passages
    Given an indexed library where more than twenty passages reach the threshold
    When the agent requests 50 passages
    Then the agent receives 20 candidate passages
    And the result is not marked short

  @givn.added @wip
  Scenario: A retrieval never changes the library
    Given the library metadata and the index state are recorded
    When the agent retrieves passages and widens context around one of them
    Then the library metadata and the index state are unchanged
