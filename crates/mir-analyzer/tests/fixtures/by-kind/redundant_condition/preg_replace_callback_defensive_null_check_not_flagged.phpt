===description===
FP-P17: preg_replace_callback's regex-error `null` is deliberately stripped from
the inferred type (real code rarely checks for it), but code that DOES defensively
check for it — the vlucas/phpdotenv `Util\Str` pattern — must not get a false
ImpossibleIdenticalComparison or RedundantCondition on its own guard.
===config===
suppress=UnusedParam,MixedArrayAccess
===file===
<?php

function replace_dates(string $pattern, string $subject): string {
    $result = preg_replace_callback(
        $pattern,
        fn($m) => $m[1],
        $subject
    );
    if ($result === null) {
        throw new \RuntimeException('invalid regex');
    }
    return $result;
}
===expect===
