===description===
A function-level suppression's body extension must stop at that function's
own closing brace — the very next function's identical diagnostic must
still be reported.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    new NoSuchClass();
}
function g(): void {
    new NoSuchClass();
}
===expect===
UndefinedClass@7:8-7:19: Class NoSuchClass does not exist
