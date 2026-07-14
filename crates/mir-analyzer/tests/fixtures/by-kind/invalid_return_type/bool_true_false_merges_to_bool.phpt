===description===
A function with no return-type hint that returns `true` from one branch and
`false` from another must have its inferred return type merged into `bool`
during union construction, not left as the decomposed `true|false` — same
principle as `null|bool` when a third `null` branch is added.
===file===
<?php
function test_true_false(): int {
    return rand(0, 1) ? true : false;
}

function test_null_true_false(): int {
    $r = rand(0, 2);
    return $r === 0 ? null : ($r === 1 ? true : false);
}
===expect===
InvalidReturnType@3:4-3:37: Return type 'bool' is not compatible with declared 'int'
InvalidReturnType@8:4-8:55: Return type 'null|bool' is not compatible with declared 'int'
