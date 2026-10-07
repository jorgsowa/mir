===description===
An array literal with a non-literal key falls back to a generic array; any
non-spread element still makes it non-empty. Only spreads may contribute nothing.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Holder {
    /** @var non-empty-array<string, int> */
    private array $counts;

    public function __construct(string $key) {
        $this->counts = [$key => 1];
    }
}

/** @param list<int> $items */
function literals(string $k, int $i, array $items): void {
    $a = [$k => 1];
    /** @mir-check $a is non-empty-array<string, 1> */
    $b = [$k => 1, 'x' => 2];
    /** @mir-check $b is non-empty-array<string, 1|2> */
    $c = [$i => 'v', ...$items];
    /** @mir-check $c is non-empty-array<int, "v"|int> */
    $d = [...$items, ...$items];
    /** @mir-check $d is array<int, int> */
    $_ = [$a, $b, $c, $d];
}
