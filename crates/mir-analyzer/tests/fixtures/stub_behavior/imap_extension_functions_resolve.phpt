===description===
FP-I1: the `imap` PECL extension (imap_open, IMAP_* constants, ...) had no
vendored stubs/ dir despite PhpStormStubsMap.php already listing every
entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable,MixedAssignment,MixedArgument,MissingReturnType
===file===
<?php

function openMailbox(string $mailbox, string $user, string $password) {
    return imap_open($mailbox, $user, $password, IMAP_GC_ELT);
}
===expect===
