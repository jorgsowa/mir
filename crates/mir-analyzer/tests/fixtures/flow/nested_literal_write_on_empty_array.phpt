===description===
A literal key followed by deeper steps on an empty array builds the nested value instead of widening to array<string, ...>
===file===
<?php
/** @return array{basic: list<int>} */
function push_under_key(): array {
    $d = [];
    $d['basic'][] = 1;
    /** @mir-check $d is array{'basic': non-empty-list<1>} */
    return $d;
}

function deep_literal_path(): void {
    $d = [];
    $d['a']['b']['c'] = 1;
    /** @mir-check $d is array{'a': array{'b': array{'c': 1}}} */
    echo json_encode($d);
}

function dynamic_key_under_literal(string $k): void {
    $d = [];
    $d['a'][$k] = 1;
    /** @mir-check $d is array{'a': array<string, 1>} */
    echo json_encode($d);
}

function sibling_keys_kept(): void {
    $d = ['x' => 1];
    $d['a']['b'] = 1;
    $d['a']['c'][] = 2;
    /** @mir-check $d is array{'x': 1, 'a': array{'b': 1, 'c': non-empty-list<2>}} */
    echo json_encode($d);
}

function repeated_push_merges(): void {
    $d = [];
    $d['a'][] = 1;
    $d['a'][] = 2;
    /** @mir-check $d is array{'a': non-empty-list<1|2>} */
    echo json_encode($d);
}

/** @return array{k?: list<int>} */
function conditional_write(bool $f): array {
    $d = [];
    if ($f) {
        $d['k'][] = 1;
    }
    /** @mir-check $d is array{'k': non-empty-list<1>}|array{} */
    return $d;
}

/**
 * @param list<int> $xs
 * @return array{k?: list<int>}
 */
function loop_write(array $xs): array {
    $d = [];
    foreach ($xs as $x) {
        $d['k'][] = $x;
    }
    /** @mir-check $d is array{'k': non-empty-list<int>}|array{} */
    return $d;
}

/** @return array{k: list<int>} */
function both_branches_write(bool $f): array {
    $d = [];
    if ($f) {
        $d['k'][] = 1;
    } else {
        $d['k'][] = 2;
    }
    return $d;
}

/** @param array<string, int> $o */
function generic_array_stays_generic(array $o): void {
    $o['a'][] = 1;
    /** @mir-check $o is array<string, int|non-empty-list<1>> */
    echo json_encode($o);
}
