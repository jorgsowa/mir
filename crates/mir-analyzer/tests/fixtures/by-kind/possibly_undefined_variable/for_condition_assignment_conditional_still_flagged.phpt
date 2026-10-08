===description===
Variables assigned in the body or step are not defined by the condition, so they
stay possibly undefined after the loop.
===file===
<?php
function bodyAssigned(int $n): void {
    for ($i = 0; $i < $n; $i++) {
        $seen = $i;
    }
    echo $seen;
}

function stepAssigned(int $n): void {
    for ($i = 0; $i < $n; $last = $i, $i++) {
    }
    echo $last;
}
===expect===
PossiblyUndefinedVariable@6:9-6:14: Variable $seen might not be defined
PossiblyUndefinedVariable@12:9-12:14: Variable $last might not be defined
