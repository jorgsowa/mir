===description===
A named function-level suppression whose only matching issue lands deep in
the body (not on the signature line) must count as used — no spurious
UnusedSuppress.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    echo 1;
    echo 2;
    new NoSuchClass();
}
===expect===
