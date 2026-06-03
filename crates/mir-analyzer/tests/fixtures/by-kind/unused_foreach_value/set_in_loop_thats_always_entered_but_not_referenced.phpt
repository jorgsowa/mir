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
    }
    return 4;
}
===expect===
UnusedVariable@7:9-7:14: Variable $last is never read
