===description===
`$arr !== [LIT]` against a sealed single-key shape removes `LIT` from that key. A literal with more than one element, a different key, or the positive branch leaves the type unchanged.
===file===
<?php
/** @param list{string} $arr */
function takesStringList(array $arr): void { echo $arr[0]; }

/** @param array{name: string} $arr */
function takesName(array $arr): void { echo $arr['name']; }

/** @param array{0: string, 1: string} $arr */
function takesPair(array $arr): void { echo $arr[0], $arr[1]; }

function caller(string|null $val): void {
    $arr = [$val];
    if ($arr === [null]) {
        return;
    }
    takesStringList($arr);
}

function flipped(string|null $val): void {
    $arr = [$val];
    if ([null] !== $arr) {
        takesStringList($arr);
    }
}

function named(string|false $val): void {
    $arr = ['name' => $val];
    if ($arr !== ['name' => false]) {
        takesName($arr);
    }
}

function unguarded(string|null $val): void {
    $arr = [$val];
    takesStringList($arr);
//                  ^^^^ PossiblyInvalidArgument: Argument $arr of takesStringList() expects 'array{0: string}', possibly different type 'array{0: string|null}' provided
}

function positive(string|null $val): void {
    $arr = [$val];
    if ($arr === [null]) {
        takesStringList($arr);
//                      ^^^^ PossiblyInvalidArgument: Argument $arr of takesStringList() expects 'array{0: string}', possibly different type 'array{0: string|null}' provided
    }
}

function otherKey(string|null $val): void {
    $arr = [$val];
    if ($arr !== [1 => null]) {
        takesStringList($arr);
//                      ^^^^ PossiblyInvalidArgument: Argument $arr of takesStringList() expects 'array{0: string}', possibly different type 'array{0: string|null}' provided
    }
}

function twoKeys(string|null $a, string|null $b): void {
    $arr = [$a, $b];
    if ($arr !== [null, null]) {
        takesPair($arr);
//                ^^^^ PossiblyInvalidArgument: Argument $arr of takesPair() expects 'array{0: string, 1: string}', possibly different type 'array{0: string|null, 1: string|null}' provided
    }
}
