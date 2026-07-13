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
}
===expect===
PossiblyUndefinedVariable@6:11-6:18: Variable $result might not be defined
