===description===
MissingReturnType fires for top-level functions without a native hint or a
docblock @return; either declaration form satisfies it.
===file===
<?php
function noReturnType($x) {
//       ^^^^^^^^^^^^ MissingReturnType: Function noReturnType() has no return type annotation
//                    ^^ MissingParamType: Parameter $x of noReturnType() has no type annotation
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
