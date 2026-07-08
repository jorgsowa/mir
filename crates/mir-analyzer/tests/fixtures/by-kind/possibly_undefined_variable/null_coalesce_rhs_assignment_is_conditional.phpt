===description===
An assignment inside the RHS of `??` only runs when the LHS is null/undefined
— it must be treated as a conditional assignment, not an unconditional one.
===file===
<?php
function foo(): ?int { return null; }
function bar(): int { return 1; }

function test(): void {
    $x = foo() ?? ($y = bar());
    echo $x;
    echo $y;
}
===expect===
PossiblyUndefinedVariable@8:9-8:11: Variable $y might not be defined
