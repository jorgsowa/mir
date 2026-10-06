===description===
A variable-variable write (`${$name} = ...`) defines names only known at runtime, so variables in that scope are not reported as unused.
===file===
<?php
function define(): void {
    foreach (['a', 'b'] as $name) {
        ${$name} = 1;
    }
}

function branch(bool $c, string $k): void {
    if ($c) {
        ${$k} = 1;
    }
    $other = 2;
}
