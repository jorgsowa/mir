===description===
MissingReturnType fires for top-level functions without a native hint or a
docblock @return; either declaration form satisfies it.
===file===
<?php
function noReturnType($x) {
    return $x;
}

function hinted(): int {
    return 1;
}

/**
 * @return string
 */
function docTyped() {
    return 'x';
}
===expect===
MissingReturnType@2:10-2:22: Function noReturnType() has no return type annotation
MissingParamType@2:23-2:25: Parameter $x of noReturnType() has no type annotation
