===description===
FP: `break N` (N > 1) was always recorded against the innermost loop's
break-context bucket instead of the loop N levels out, so code after the
OUTER loop was wrongly reported unreachable even though `break 2` reaches it.
===config===
suppress=MixedArgument,MixedAssignment
===file===
<?php
function foo(array $matrix): void {
    while (true) {
        foreach ($matrix as $cell) {
            if ($cell === 0) {
                break 2;
            }
        }
    }
    echo "after";
}
===expect===
