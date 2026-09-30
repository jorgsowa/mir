===description===
Only $GLOBALS was recognized as external mutable state for the purity
check — $_SESSION/$_ENV/etc. are exactly the same shape (reading OR
writing them depends on/mutates state outside the function) but were
completely unrecognized, for both a read and a write.
===config===
suppress=MixedArrayAccess,MixedReturnStatement,MixedAssignment
===file===
<?php
/** @pure */
function readSession(): int {
    return $_SESSION['x'];
//         ^^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $x in a @pure function
}

/** @pure */
function writeSession(int $n): void {
    $_SESSION['x'] = $n;
//  ^^^^^^^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $x in a @pure function
}

/** @pure */
function readEnv(): string {
    return $_ENV['HOME'];
//         ^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $HOME in a @pure function
}
===expect===
