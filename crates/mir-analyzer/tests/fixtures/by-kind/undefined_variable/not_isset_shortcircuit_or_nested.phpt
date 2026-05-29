===description===
!isset short-circuit with || operator — nested in compound conditions
Narrowing applies correctly when !isset() check is nested within && and || chains
===file===
<?php
function someFunc(): bool { return true; }
if (someFunc() && (!isset($x) || $x->method())) {
    // After fix: $x should be narrowed in RHS of !isset($x) ||
}
===expect===
MixedMethodCall@3:34-3:46: Method method() called on mixed type
