===description===
A call may run a closure that writes null to a by-reference capture, so a
null check does not stay decided across it; without a call it still does.
===file===
<?php
function afterCall(?string $x): void {
    $f = function () use (&$x): void { $x = null; };
    if ($x !== null) { $f(); if ($x === null) {} }
}

function withoutCall(?string $x): void {
    $f = function () use (&$x): void { $x = null; };
    if ($x !== null) { if ($x === null) {} }
//                         ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
    $f();
}
