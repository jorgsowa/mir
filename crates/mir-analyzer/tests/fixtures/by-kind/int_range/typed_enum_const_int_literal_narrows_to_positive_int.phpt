===description===
The typed-const literal-narrowing gap was duplicated verbatim in the enum collector: a
typed constant declared on an enum lost the same literal precision as the class case.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;

    private const int DEFAULT_COUNT = 4;

    public function counts(): void {
        baz([self::DEFAULT_COUNT]);
    }
}

/** @param list<positive-int> $counts */
function baz(array $counts): void {}

===expect===
