===description===
P2: A `: never` function that only conditionally diverges (some paths fall through)
must be flagged. A function that diverges on ALL paths is fine.
===file===
<?php

// Flagged: only some paths throw
function conditional_throw(bool $flag): never {
    if ($flag) {
        throw new RuntimeException("throws sometimes");
    }
}

// Flagged: only some paths exit
function conditional_exit(bool $flag): never {
    if ($flag) {
        exit(1);
    }
}

// Not flagged: all paths diverge
function always_throws(bool $flag): never {
    if ($flag) {
        throw new RuntimeException("path A");
    } else {
        throw new LogicException("path B");
    }
}

// Not flagged: calls another never function
function calls_never(bool $flag): never {
    always_throws($flag);
}
===expect===
InvalidReturnType@4:46-8:1: Return type 'void' is not compatible with declared 'never'
InvalidReturnType@11:45-15:1: Return type 'void' is not compatible with declared 'never'
