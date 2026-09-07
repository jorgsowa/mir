===description===
A variable-variable assignment suppresses later UndefinedVariable, including across branch merges, but does not suppress reads that precede any dynamic definition.
===file===
<?php
function direct_name(string $key): void {
    ${$key} = 1;
    echo $b;            // ok: a dynamic def may have created $b
}

function in_branch(bool $c, string $key): void {
    if ($c) {
        ${$key} = 1;
    }
    echo $z;            // ok: $z may have been defined dynamically on one path
}

function reported_before_def(): void {
    echo $early;        // still an error: nothing dynamic has happened yet
    ${"x"} = 1;
}

===expect===
UndefinedVariable@15:9-15:15: Variable $early is not defined
