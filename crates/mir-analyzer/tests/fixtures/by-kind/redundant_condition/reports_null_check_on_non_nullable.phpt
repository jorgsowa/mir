===description===
reports null check on non nullable
===file===
<?php
function f(string $x): void {
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
}
