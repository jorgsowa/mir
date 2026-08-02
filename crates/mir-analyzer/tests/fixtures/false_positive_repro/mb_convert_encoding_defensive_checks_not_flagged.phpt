===description===
FP-P17: mb_convert_encoding's on-error `false` is deliberately stripped from the
inferred type (real code rarely checks for it), but code that DOES defensively
check or cast against it must not get a false ImpossibleIdenticalComparison or
RedundantCast on its own guard.
===config===
suppress=UnusedParam
===file===
<?php

function convert_checked(string $s): string {
    $result = mb_convert_encoding($s, 'UTF-8', 'ISO-8859-1');
    if ($result === false) {
        throw new \RuntimeException('invalid encoding');
    }
    return $result;
}

function convert_cast(string $s): string {
    return (string) mb_convert_encoding($s, 'UTF-8', 'ISO-8859-1');
}
===expect===
