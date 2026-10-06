===description===
A nested literal-key write adds the key to the inner shape instead of widening the whole array
===file===
<?php
/** @return array{fields: array<string, bool>} */
function emptyInner(): array {
    $a = ['fields' => []];
    $a['fields']['k'] = true;
    /** @mir-check $a is array{'fields': array{'k': true}} */
    return $a;
}

function populatedInner(): void {
    $a = ['fields' => ['x' => 1]];
    $a['fields']['k'] = true;
    /** @mir-check $a is array{'fields': array{'x': 1, 'k': true}} */
    $a;
}

function siblingsKept(): void {
    $a = ['fields' => [], 'other' => 1];
    $a['fields']['k'] = true;
    /** @mir-check $a is array{'fields': array{'k': true}, 'other': 1} */
    $a;
}

function threeLevels(): void {
    $a = ['one' => ['two' => []]];
    $a['one']['two']['k'] = 1;
    /** @mir-check $a is array{'one': array{'two': array{'k': 1}}} */
    $a;
}

function conditionalWrite(bool $c): void {
    $a = ['fields' => []];
    if ($c) {
        $a['fields']['k'] = true;
    }
    /** @mir-check $a is array{'fields': array{'k': true}}|array{'fields': array{}} */
    $a;
}

/** @param list<string> $xs */
function loopWrite(array $xs): void {
    $a = ['fields' => []];
    foreach ($xs as $x) {
        $a['fields']['k'] = $x;
    }
    /** @mir-check $a is array{'fields': array{'k': string}}|array{'fields': array{}} */
    $a;
}

/** @param array<string, int> $map */
function nonShapeInnerStaysGeneric(array $map): void {
    $a = ['fields' => $map];
    $a['fields']['k'] = 1;
    /** @mir-check $a is array<string, array<"k", 1>|array<string, int>> */
    $a;
}

/** @param array{fields: array<string, int>} $shape */
function takesFields(array $shape): int {
    return count($shape);
}

function validValueIsAccepted(): void {
    $a = ['fields' => []];
    $a['fields']['k'] = 1;
    takesFields($a);
}

function wrongValueIsStillRejected(): void {
    $a = ['fields' => []];
    $a['fields']['k'] = 'text';
    takesFields($a);
//              ^^ InvalidArgument: Argument $shape of takesFields() expects 'array{'fields': array<string, int>}', got 'array{'fields': array{'k': "text"}}'
}
===expect===
