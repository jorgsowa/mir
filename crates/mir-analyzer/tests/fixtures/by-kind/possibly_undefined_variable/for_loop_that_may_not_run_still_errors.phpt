===description===
A for loop whose first check fails, or isn't decided by literals, may skip its body
===file===
<?php
function never(): int {
    for ($i = 0; $i < 0; $i++) {
        $v = $i;
    }
    return $v;
//         ^^ PossiblyUndefinedVariable: Variable $v might not be defined
}
function upTo(int $n): int {
    for ($i = 0; $i < $n; $i++) {
        $v = $i;
    }
    return $v;
//         ^^ PossiblyUndefinedVariable: Variable $v might not be defined
}
