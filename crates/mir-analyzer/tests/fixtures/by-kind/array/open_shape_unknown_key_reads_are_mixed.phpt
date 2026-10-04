===description===
An open shape (bare `array` plus a literal-key write, or a docblock `...` shape) holds anything
under keys it doesn't list, so unknown-key reads, iteration, `array_values` and spreads yield
`mixed`; listed keys stay precise and closed shapes keep their known-values behavior.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function untyped(): array { return []; }
function take_string(string $s): void {}
function take_ints(int ...$n): void {}

function literal_unknown_key(): void {
    $p = untyped();
    $p['flag'] = true;
    /** @mir-check $p['flag'] is true */
    $_ = $p['flag'];
    /** @mir-check $p['type'] is mixed */
    $_ = $p['type'];
    echo $p['type'] === 'a';
    take_string($p['other']);
}

function dynamic_key(string $k): void {
    $p = untyped();
    $p['flag'] = true;
    /** @mir-check $p[$k] is mixed */
    $_ = $p[$k];
    take_string($p[$k]);
}

function iteration_and_values(): void {
    $p = untyped();
    $p['flag'] = true;
    /** @mir-check array_values($p) is list<mixed> */
    $_ = array_values($p);
    foreach ($p as $k => $v) {
        /** @mir-check $v is mixed */
        $_ = $v;
        take_string($v);
    }
}

function spreads(): void {
    $p = untyped();
    $p['n'] = 1;
    take_ints(...$p);
    $merged = [...$p, 'x' => 'y'];
    /** @mir-check $merged['other'] is mixed */
    $_ = $merged['other'];
}

/** @param array{id: int, ...} $row */
function docblock_open_shape(array $row): void {
    /** @mir-check $row['id'] is int */
    $_ = $row['id'];
    /** @mir-check $row['extra'] is mixed */
    $_ = $row['extra'];
    take_string($row['extra']);
}

/** @param array{id: int} $row */
function closed_shape_unchanged(array $row): void {
    /** @mir-check array_values($row) is non-empty-list<int> */
    $_ = array_values($row);
    $row['missing'];
}
===expect===
NonExistentArrayOffset@59:9-59:18: Array offset 'missing' does not exist
