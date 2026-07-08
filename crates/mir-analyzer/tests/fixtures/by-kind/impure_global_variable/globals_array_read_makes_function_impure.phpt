===description===
Reading $GLOBALS['x'] reaches the same external mutable state as
`global $x;`, but only the `global` statement was ever checked — a plain
read through the superglobal array bypassed the purity check entirely.
===config===
suppress=MixedArrayAccess,MixedReturnStatement
===file===
<?php
/** @pure */
function test(): int {
    return $GLOBALS['x'];
}
===expect===
ImpureGlobalVariable@4:11-4:24: Using global variable $x in a @pure function
