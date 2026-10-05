===description===
A new literal key written inside a loop grows the keyed shape in place instead of generalizing it to a generic array.
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array{a: int, b?: string} */
function viaForeach(array $xs): array {
    $r = ['a' => 1];
    foreach ($xs as $x) {
        $r['b'] = 'x';
    }
    return $r;
}

/** @return array{a: int, b?: string} */
function viaWhile(): array {
    $r = ['a' => 1];
    while (rand() > 0) {
        $r['b'] = 'x';
    }
    return $r;
}

/** @return array{a: int, b?: string, c?: bool} */
function severalKeys(array $xs): array {
    $r = ['a' => 1];
    foreach ($xs as $x) {
        $r['b'] = 'x';
        if ($x) {
            $r['c'] = true;
        }
    }
    return $r;
}

/** @return array{a: int, b?: string} */
function nestedLoops(array $xs): array {
    $r = ['a' => 1];
    foreach ($xs as $x) {
        foreach ($x as $y) {
            $r['b'] = 'x';
        }
    }
    return $r;
}

/** @return array{a: int, b?: string} */
function guaranteedLoop(): array {
    $r = ['a' => 1];
    for ($i = 0; $i < 3; $i++) {
        $r['b'] = 'x';
    }
    return $r;
}

function overwriteInLoop(array $xs): void {
    $r = ['a' => 1];
    foreach ($xs as $x) {
        $r['a'] = 2;
    }
    /** @mir-check $r is array{a: 2}|array{a: 1} */
    $r;
}

function conditionalKeyInLoop(array $xs): void {
    $r = ['a' => 1];
    foreach ($xs as $x) {
        $r['b'] = 'x';
    }
    /** @mir-check $r is array{a: 1, b: 'x'}|array{a: 1} */
    $r;
}

function nonLiteralKeyStillGeneralizes(array $xs, string $k): void {
    $r = ['a' => 1];
    foreach ($xs as $x) {
        $r[$k] = 'x';
    }
    /** @mir-check $r is array<string, 'x'|1>|array{a: 1} */
    $r;
}
function writeAfterBranchThatAddedKey(bool $c): void {
    $r = ['a' => 1];
    if ($c) {
        $r['b'] = 1;
    }
    $r['b'] = 'x';
    /** @mir-check $r is array{a: 1, b: 'x'} */
    $r;
}
===expect===
