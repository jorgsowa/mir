===description===
A `@return literal-string` function may return literals and constants, not text built from its arguments.
===file===
<?php
/** @return literal-string */
function greeting(bool $formal): string {
    return $formal ? 'Good morning' : 'Hi';
}

/** @return literal-string */
function personalGreeting(string $name): string {
    return 'Hi ' . $name;
//  ^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'non-empty-string' is not compatible with declared 'literal-string'
}

/** @return literal-int */
function retries(): int {
    return 3;
}
