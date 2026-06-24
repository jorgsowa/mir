===description===
true is a subtype of bool — bool === true and true === true should not fire.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_bool(bool $b): void {
    if ($b === true) {}
    if ($b === false) {}
}

function test_true(true $x): void {
    if ($x === true) {}
}
===expect===
