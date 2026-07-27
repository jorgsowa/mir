===description===
A function-level suppression covers every matching-kind diagnostic anywhere
in the body, not just the first one — multiple independent occurrences of
the same named kind, several lines apart, are all suppressed.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    new NoSuchClassA();
    echo 'between';
    new NoSuchClassB();
}
===expect===
