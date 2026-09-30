===description===
Set in loop thats always entered but not referenced
===file===
<?php
/**
 * @param non-empty-array<int> $a
 */
function getLastNum(array $a): int {
    foreach ($a as $num) {
        $last = $num;
//      ^^^^^ UnusedVariable: Variable $last is never read
    }
    return 4;
}
===expect===
