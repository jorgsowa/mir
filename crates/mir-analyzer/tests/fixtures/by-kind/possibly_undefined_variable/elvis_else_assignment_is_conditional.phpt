===description===
An assignment inside the else-arm of a short ternary (`?:`) only runs when
the condition is falsy — it must be treated as a conditional assignment,
not an unconditional one.
===file===
<?php
function cond(): bool { return false; }
function def(): int { return 1; }

function test(): void {
    $x = cond() ?: ($y = def());
    echo $x;
    echo $y;
}
===expect===
PossiblyUndefinedVariable@8:9-8:11: Variable $y might not be defined
