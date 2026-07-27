===description===
A `@psalm-suppress` docblock above a function declaration previously only
covered the signature line itself, never the function's own body — the
documented Psalm/PHPStan semantics is that a function-level suppression
covers every diagnostic inside that function.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    new NoSuchClass();
}
===expect===
