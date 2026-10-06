===description===
A bare (whole-array, non-indexed) superglobal read/write bypassed the
purity check entirely -- only the indexed shape ($_SERVER['x']) was
ever checked, in arrays.rs/assign_to_target's ArrayAccess arm. Reading
or overwriting the WHOLE superglobal array is the same external mutable
state, just without an index.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @pure */
function dumpServer(): array {
    return $_SERVER;
//         ^^^^^^^^ ImpureGlobalVariable: Using global variable $_SERVER in a @pure function
}

/** @pure */
function resetSession(): void {
    $_SESSION = [];
//  ^^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $_SESSION in a @pure function
}
