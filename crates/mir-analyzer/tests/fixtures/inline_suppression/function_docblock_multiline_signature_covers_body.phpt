===description===
A function-level suppression still reaches the body when the function's own
signature spans multiple physical lines (one parameter per line) before the
opening brace — the body-extension scan must pass through the parameter
list unaffected.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(
    int $a,
    string $b
): void {
    echo $a . $b;
    new NoSuchClass();
}
===expect===
