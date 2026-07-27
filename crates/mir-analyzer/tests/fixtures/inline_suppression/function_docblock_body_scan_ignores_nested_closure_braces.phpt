===description===
The body-extension scan must track brace depth through a nested closure
inside the function body and stop at the OUTER function's own closing
brace — not the inner closure's — so the diagnostic right after the
function is still reported.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    $cb = function (): void {
        new NoSuchClassInside();
    };
    $cb();
}
new NoSuchClassOutside();
===expect===
UndefinedClass@9:4-9:22: Class NoSuchClassOutside does not exist
