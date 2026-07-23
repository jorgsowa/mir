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
}

/** @pure */
function writeSession(int $n): void {
    $_SESSION['x'] = $n;
}

/** @pure */
function readEnv(): string {
    return $_ENV['HOME'];
}
===expect===
ImpureGlobalVariable@4:11-4:25: Using global variable $x in a @pure function
ImpureGlobalVariable@9:4-9:23: Using global variable $x in a @pure function
ImpureGlobalVariable@14:11-14:24: Using global variable $HOME in a @pure function
