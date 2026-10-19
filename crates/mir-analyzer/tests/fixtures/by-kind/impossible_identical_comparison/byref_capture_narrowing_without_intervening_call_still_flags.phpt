===description===
Without a call between the narrowing and the comparison the literal is still
known, and by-value captures cannot be rewritten by a call.
===file===
<?php
function noCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { if ($x === 'z') {} }
    $f();
}

function byValue(string $x): void {
    $f = function () use ($x): void { $x = 'z'; };
    if ($x === 'a') { $f(); if ($x === 'z') {} }
}

function unrelatedVariable(string $x, string $y): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($y === 'a') { $f(); if ($y === 'z') {} }
}

function reassignedAfterCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    $f();
    $x = 'a';
    if ($x === 'z') {}
}

function writeNotCaptured(string $x): void {
    $f = function (): void { $x = 'z'; };
    if ($x === 'a') { $f(); if ($x === 'z') {} }
}
===expect===
ImpossibleIdenticalComparison@4:26-4:36: '===' between '"a"' and '"z"' is always false — these types can never be identical
ImpossibleIdenticalComparison@10:32-10:42: '===' between '"a"' and '"z"' is always false — these types can never be identical
ImpossibleIdenticalComparison@15:32-15:42: '===' between '"a"' and '"z"' is always false — these types can never be identical
ImpossibleIdenticalComparison@22:8-22:18: '===' between '"a"' and '"z"' is always false — these types can never be identical
ImpossibleIdenticalComparison@27:32-27:42: '===' between '"a"' and '"z"' is always false — these types can never be identical
