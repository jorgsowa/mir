===description===
reports not null check on non nullable
===file===
<?php
function f(string $x): void {
    if ($x !== null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'string' and 'null' is always true — these types can never be identical
}
