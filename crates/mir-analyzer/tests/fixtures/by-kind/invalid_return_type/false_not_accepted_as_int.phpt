===description===
`false` and `false|null` returned where `int` is declared must be flagged;
`remove_false(TFalse)` returns empty which vacuously passes subtype checks
without a guard.
===file===
<?php
function test_false(): int {
    return false;
}

function test_null_false(): int {
    return rand(0, 1) ? null : false;
}
===expect===
InvalidReturnType@3:4-3:17: Return type 'false' is not compatible with declared 'int'
InvalidReturnType@7:4-7:37: Return type 'null|false' is not compatible with declared 'int'
