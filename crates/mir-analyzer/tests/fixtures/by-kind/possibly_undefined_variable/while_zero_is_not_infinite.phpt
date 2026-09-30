===description===
while(0) is falsy, not an infinite loop like while(1) — a variable
assigned only inside its body is still possibly-undefined after the loop.
===file===
<?php
function foo(): int {
    while (0) {
        $result = 1;
    }
    return $result;
//         ^^^^^^^ PossiblyUndefinedVariable: Variable $result might not be defined
}
===expect===
