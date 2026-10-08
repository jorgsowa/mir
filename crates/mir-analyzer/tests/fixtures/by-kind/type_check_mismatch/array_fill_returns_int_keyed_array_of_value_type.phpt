===description===
`array_fill` with a possibly-empty count returns `array<int, TValue>` with literal values
widened, so the filled array is assignable to a typed property or
parameter and its value type follows the argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Entry {}

final class Table {
    /** @var array<int, Entry|null> */
    private array $buckets;

    public function __construct(int $size = 0) {
        $this->buckets = array_fill(0, $size, null);
    }
}

/** @param array<int, int> $a */
function sum(array $a): int {
    return array_sum($a);
}

function types(int $n): void {
    $unknownCount = array_fill(0, $n, 'x');
    /** @mir-check $unknownCount is array<int, string> */
    $entries = array_fill(5, $n, new Entry());
    /** @mir-check $entries is array<int, Entry> */
    $counts = array_fill(0, $n, 0);
    /** @mir-check $counts is array<int, int> */
    $counts[2] = 7;
    $flags = array_fill(0, $n, false);
    /** @mir-check $flags is array<int, bool> */
    echo count($flags);
    echo sum($counts), count($unknownCount), count($entries);
}

function wrongValue(int $n): void {
    sum(array_fill(0, $n, 'x'));
//      ^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $a of sum() expects 'array<int, int>', got 'array<int, string>'
}
