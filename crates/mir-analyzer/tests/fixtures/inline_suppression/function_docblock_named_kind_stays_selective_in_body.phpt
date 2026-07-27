===description===
A named function-level suppression only silences its own kind inside the
body — a different kind on the same body line is unaffected.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    new NoSuchClass(noSuchFunc());
}
===expect===
UndefinedFunction@4:20-4:32: Function noSuchFunc() is not defined
