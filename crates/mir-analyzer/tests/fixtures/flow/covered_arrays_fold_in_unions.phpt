===description===
A union drops an array another member of the same kind covers, so repeated conditional wrapping and `array_merge` chains grow linearly; shapes with other keys, optionality, objects or an int against a float stay separate.
===file===
<?php
class Foo {}
class Bar {}

function sequentialConditionalWrapping(int $n): void {
    $v = 0;
    if ($n === 1) { $v = [$v]; }
    if ($n === 2) { $v = [$v]; }
    if ($n === 3) { $v = [$v]; }
    /** @mir-check $v is array{0: array{0: array{0: 0}|0}|0}|int */
    var_dump($v);
}

function sequentialConditionalKeyedWrapping(int $n): void {
    $v = ['leaf' => 'x'];
    if ($n === 1) { $v = ['node' => $v]; }
    if ($n === 2) { $v = ['node' => $v]; }
    /** @mir-check $v is array{'node': array{'node': array{'leaf': "x"}}|array{'leaf': "x"}}|array{'leaf': "x"} */
    var_dump($v);
}

function arrayMergeChain(): void {
    $r = ['depth' => 0];
    $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]);
    /** @mir-check $r is non-empty-array<"depth"|"next", 0|array{'depth': 0}|non-empty-array<"depth"|"next", 0|array{'depth': 0}|non-empty-array<"depth"|"next", 0|array{'depth': 0}|non-empty-array<"depth"|"next", 0|array{'depth': 0}>>>> */
    var_dump($r);
}

function literalShapeFoldsIntoBase(bool $b, int $n): void {
    $a = $b ? ['id' => 1, 'name' => 'x'] : ['id' => $n, 'name' => 'x'];
    /** @mir-check $a is array{'id': int, 'name': "x"} */
    var_dump($a);
}

function intShapeDoesNotFoldIntoFloat(bool $b, float $f): void {
    $a = $b ? ['v' => 1] : ['v' => $f];
    /** @mir-check $a is array{'v': 1}|array{'v': float} */
    var_dump($a);
    if (is_int($a['v'])) { echo "int"; }
}

function differentKeysStaySeparate(bool $b): void {
    $a = $b ? ['ok' => true, 'value' => 1] : ['ok' => false, 'error' => 'e'];
    /** @mir-check $a is array{'ok': true, 'value': 1}|array{'ok': false, 'error': "e"} */
    var_dump($a);
    if ($a['ok']) { echo $a['value']; } else { echo $a['error']; }
}

function optionalKeyStaysSeparate(bool $b, int $n): void {
    /** @var array{id?: int} $opt */
    $opt = [];
    $a = $b ? $opt : ['id' => $n];
    /** @mir-check $a is array{'id'?: int}|array{'id': int} */
    var_dump($a);
}

function objectShapesStaySeparate(bool $b): void {
    $a = $b ? ['o' => new Foo] : ['o' => new Bar];
    /** @mir-check $a is array{'o': Foo}|array{'o': Bar} */
    var_dump($a);
}

/** @param array<string, Bar> $bars */
function objectShapeNotCoveredByGenericArray(bool $b, array $bars): void {
    $a = $b ? ['k' => ['x' => new Foo]] : ['k' => $bars];
    /** @mir-check $a is array{'k': array{'x': Foo}}|array{'k': array<string, Bar>} */
    var_dump($a);
}
