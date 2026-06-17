===description===
`bool` returned where `int` is declared must be flagged; `remove_false` on
`bool` should yield `true`, not an empty type that vacuously passes the check.
===file===
<?php
function test(): int {
    /** @var bool $b */
    $b = true;
    return $b;
}
===expect===
InvalidReturnType@5:4-5:14: Return type 'bool' is not compatible with declared 'int'
