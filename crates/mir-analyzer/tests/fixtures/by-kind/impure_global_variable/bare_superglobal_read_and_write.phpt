===description===
A bare (whole-array, non-indexed) superglobal read/write bypassed the
purity check entirely -- only the indexed shape ($_SERVER['x']) was
ever checked, in arrays.rs/assign_to_target's ArrayAccess arm. Reading
or overwriting the WHOLE superglobal array is the same external mutable
state, just without an index.
===config===
suppress=MixedReturnStatement
===file===
<?php
/** @pure */
function dumpServer(): array {
    return $_SERVER;
}

/** @pure */
function resetSession(): void {
    $_SESSION = [];
}
===expect===
ImpureGlobalVariable@4:11-4:19: Using global variable $_SERVER in a @pure function
ImpureGlobalVariable@9:4-9:18: Using global variable $_SESSION in a @pure function
