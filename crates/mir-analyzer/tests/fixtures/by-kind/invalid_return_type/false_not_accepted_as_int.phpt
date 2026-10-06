===description===
`false` and `false|null` returned where `int` is declared must be flagged;
`remove_false(TFalse)` returns empty which vacuously passes subtype checks
without a guard.
===file===
<?php
function test_false(): int {
    return false;
//  ^^^^^^^^^^^^^ InvalidReturnType: Return type 'false' is not compatible with declared 'int'
}

function test_null_false(): int {
    return rand(0, 1) ? null : false;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'null|false' is not compatible with declared 'int'
}
