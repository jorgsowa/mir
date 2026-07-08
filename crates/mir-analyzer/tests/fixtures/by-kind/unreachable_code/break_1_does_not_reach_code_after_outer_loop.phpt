===description===
A plain `break;` (level 1) only exits the innermost `foreach`; the outer
`while (true)` still never exits normally, so code after it is still
correctly unreachable.
===config===
suppress=MixedArgument,MixedAssignment
===file===
<?php
function foo(array $matrix): void {
    while (true) {
        foreach ($matrix as $cell) {
            if ($cell === 0) {
                break;
            }
        }
    }
    echo "after";
}
===expect===
UnreachableCode@10:4-10:17: Unreachable code detected
