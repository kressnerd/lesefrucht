@givn.delta @passage-retrieval

Feature: Passage retrieval

  The agent submits a claim or phrase to the indexed library and receives a
  candidate-passage result with cited passages, or an explicit no-evidence
  result, or an unavailability report.

  @givn.added @e2e @wip
  Scenario: Agent retrieves cited passages for a claim
    Given the library holds at least one indexed edition
    And the configured expansion translates the German term "Vertrauen" to the English term "trust"
    And an agent connected through the passage-retrieval interface
    When the agent submits the claim "Vertrauen in Organisationen" in German with scope genre=fachbuch, requesting 5 passages
    Then the agent receives a candidate-passage result
    And the result holds at least one passage and at most 5
    And the result is marked short exactly when the returned number is smaller than the requested number
    And every returned passage carries a citation to an indexed edition, a score and the expansion used
    And at least one returned citation belongs to an English edition
    And the expansion names the term "trust"
    And the passages are ordered by score, then work id in numeric order, then passage index

  @givn.added @e2e @wip
  Scenario: Agent widens context around a cited passage
    Given an agent connected through the passage-retrieval interface
    And the agent has retrieved a citation to an indexed edition
    When the agent requests wider context around the cited passage
    Then the agent receives the surrounding text with the cited passage inside it
    And the returned text between the cited start and end offsets equals the text of the cited passage
    And the cited start offset is smaller than the cited end offset
    And the bounds start at or before the cited start and end at or after the cited end
    And the bounds name the same chapter as the cited passage

  @givn.added
  Scenario: Scope restricts passages to the named genre
    Given the claim appears in indexed works of the genres fachbuch and sachbuch
    When the agent submits that claim with scope genre=fachbuch
    Then at least one returned passage comes from a work of the genre fachbuch
    And every returned citation belongs to an indexed work of the genre fachbuch
    And no returned citation belongs to a work outside the genre fachbuch

  @givn.added
  Scenario: A scope naming an unknown work yields no evidence
    Given an indexed library
    When the agent submits a claim with a scope naming the work "999"
    Then the agent receives no evidence with reason unknown work
    And no passage is returned

  @givn.added
  Scenario: No passage above the threshold yields no evidence
    Given an indexed library that holds no passage for the claim
    When the agent submits the claim
    Then the agent receives no evidence with reason no evidence
    And the result is neither a failure nor an empty success

  @givn.added
  Scenario: A stale index yields an unavailability report
    Given an index that is older than the freshness bound
    When the agent submits a claim
    Then the agent receives an unavailability report with reason stale index
    And the report is not a no-evidence result

  @givn.added
  Scenario: An unavailable index yields an unavailability report
    Given no edition is indexed
    When the agent submits a claim
    Then the agent receives an unavailability report with reason index unavailable
    And the report is not a no-evidence result

  @givn.added
  Scenario: Fewer passages than requested are returned short
    Given an indexed library where four passages reach the threshold
    When the agent requests 20 passages
    Then the result holds 4 passages
    And the result is marked short
    And no passage is padded to reach the requested number

  @givn.added
  Scenario: The result is capped at twenty passages
    Given an indexed library where more than twenty passages reach the threshold
    When the agent requests 50 passages
    Then the result holds 20 passages
    And the result is not marked short

  @givn.added @wip
  Scenario: A retrieval never changes the library
    Given the library metadata listing, the accepted glossary and the index generation are recorded
    And the last successful index time reported by the readiness probe is recorded
    When the agent retrieves passages and widens context around one of them
    Then the library metadata listing is unchanged
    And the accepted glossary is unchanged
    And the index generation and the last successful index time are unchanged

  @givn.added @wip
  Scenario: Identical requests return the same sequence
    Given an indexed library
    When the agent submits the same claim, scope and limit twice
    Then both candidate-passage results carry the same citations in the same order

  @givn.added @wip
  Scenario: A returned citation resolves to the passage it cites
    Given an indexed library whose claim is above the evidence threshold
    When the agent retrieves passages and reads the first citation with window_chars 0
    Then the returned text equals the text of the passage at the cited start and end offsets

  @givn.added @wip
  Scenario: A widened context is clamped to the chapter and the maximum window
    Given a cited passage that starts less than the maximum window after its chapter start
    When the agent requests a window wider than the configured maximum window
    Then the bounds do not extend beyond the chapter
    And the window is no wider than the configured maximum window
    And the result reports that it was truncated

  @givn.added @wip
  Scenario: An unknown citation is rejected as an invalid request
    Given an indexed library
    When the agent reads a passage with a citation the index does not hold
    Then the request fails with reason invalid citation
    And the failure is not a no-evidence result

  @givn.added @wip
  Scenario: A malformed request is rejected
    Given an agent connected through the passage-retrieval interface
    When the agent submits a request that names no claim, no language or a limit below 1
    Then the request fails as an invalid argument
    And no result is returned

  @givn.added @wip
  Scenario: Scope filters passages by language
    Given the claim appears in indexed works in German and in English
    When the agent submits that claim with scope language English
    Then at least one returned citation belongs to a work whose language is English
    And every returned citation belongs to a work whose language is English

  @givn.added @wip
  Scenario: Scope filters passages by author and year
    Given the claim appears in indexed works of two authors and two publication years
    When the agent submits that claim with a scope naming one author and one year
    Then at least one returned citation belongs to a work of that author published in that year
    And every returned citation belongs to a work of that author published in that year

  @givn.added @wip
  Scenario: An abstention names the editions that failed indexing
    Given an indexed library whose work "5" holds only an edition in a format the product cannot read
    When the agent submits a claim with a scope naming that work
    Then the agent receives no evidence with reason no evidence
    And the result names work "5" as an edition that failed indexing with reason unsupported format

  @givn.added @wip
  Scenario: An unreachable library yields an unavailability report
    Given the library cannot be reached
    When the agent submits a claim
    Then the agent receives an unavailability report with reason library unreachable
    And the report is not a no-evidence result
