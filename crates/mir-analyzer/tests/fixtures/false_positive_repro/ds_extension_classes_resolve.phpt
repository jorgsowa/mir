===description===
FP-I1: the `ds` PECL extension (Ds\Vector, Ds\Map, Ds\Set, ...) had no
vendored stubs/ dir despite PhpStormStubsMap.php already listing every
entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

use Ds\Vector;
use Ds\Map;
use Ds\Set;

function buildVector(): Vector {
    return new Vector([1, 2, 3]);
}

function buildMap(): Map {
    return new Map(['a' => 1]);
}

function buildSet(): Set {
    return new Set([1, 2, 3]);
}
===expect===
