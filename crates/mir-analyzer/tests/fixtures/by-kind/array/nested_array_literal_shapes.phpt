===description===
Nested array literals and array_map closures returning shapes keep the inner shape.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{a: array{b: int}} $s */
function wantsNested(array $s): void { echo count($s); }

/** @param list<array{id: int, name: string}> $rows */
function wantsRows(array $rows): void { echo count($rows); }

function nested(): void {
    $n = ['a' => ['b' => 1]];
    /** @mir-check $n is array{'a': array{'b': 1}} */
    wantsNested($n);
    wantsNested(['a' => ['b' => 1]]);
}

function listOfShapes(): void {
    $rows = [['id' => 1, 'name' => 'a'], ['id' => 2, 'name' => 'b']];
    wantsRows($rows);
    wantsRows([['id' => 1, 'name' => 'a']]);
}

/** @param list<int> $ids */
function mapped(array $ids): void {
    $m = array_map(fn(int $i) => ['id' => $i, 'name' => 'x'], $ids);
    /** @mir-check $m is list<array{'id': int, 'name': 'x'}> */
    wantsRows($m);
    $c = array_map(function (int $i): array { return ['id' => $i, 'name' => 'x']; }, $ids);
    /** @mir-check $c is list<array{'id': int, 'name': string}> */
    wantsRows($c);
}

/** @param array<string, int> $by */
function mappedKeyed(array $by): void {
    $m = array_map(fn(int $i) => ['n' => ['v' => $i]], $by);
    /** @mir-check $m is array<string, array{'n': array{'v': int}}> */
    echo 1;
}

/** @param list<int> $ids */
function declaredArray(array $ids): void {
    $a = array_map(fn(int $i): array => ['id' => $i, 'name' => 'x'], $ids);
    /** @mir-check $a is list<array{'id': int, 'name': string}> */
    wantsRows($a);
    $b = array_map(fn(int $i): array => ['n' => ['v' => $i, 'w' => 'x']], $ids);
    /** @mir-check $b is list<array{'n': array{'v': int, 'w': string}}> */
    $c = array_map(fn(int $i): array => [$i, 2], $ids);
    /** @mir-check $c is list<array> */
    echo 1;
}

function deep(): void {
    $d = ['x' => [['y' => [1, 2]]]];
    /** @mir-check $d is array{'x': array{0: array{'y': array{0: 1, 1: 2}}}} */
    echo 1;
}
===expect===
