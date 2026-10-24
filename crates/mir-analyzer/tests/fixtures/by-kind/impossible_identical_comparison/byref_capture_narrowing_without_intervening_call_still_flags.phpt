===description===
Without a call between the narrowing and the comparison the literal is still
known, and by-value captures cannot be rewritten by a call.
===file===
<?php
function noCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { if ($x === 'z') {} }
//                        ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
    $f();
}

function byValue(string $x): void {
    $f = function () use ($x): void { $x = 'z'; };
    if ($x === 'a') { $f(); if ($x === 'z') {} }
//                              ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
}

function unrelatedVariable(string $x, string $y): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($y === 'a') { $f(); if ($y === 'z') {} }
//                              ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
}

function reassignedAfterCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    $f();
    $x = 'a';
    if ($x === 'z') {}
//      ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
}

function writeNotCaptured(string $x): void {
    $f = function (): void { $x = 'z'; };
    if ($x === 'a') { $f(); if ($x === 'z') {} }
//                              ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"' and '"z"' is always false — these types can never be identical
}
