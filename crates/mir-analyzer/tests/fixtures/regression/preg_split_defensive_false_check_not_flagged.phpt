===description===
FP-P17: preg_split's regex-error `false` is deliberately stripped from the inferred
type (real code rarely checks for it), but code that DOES defensively check for it —
the vlucas/phpdotenv `Util\Regex::split()` pattern — must not get a false
ImpossibleIdenticalComparison on its own guard.
===config===
suppress=UnusedParam
===file===
<?php

function split_words(string $pattern, string $subject): array {
    $result = preg_split($pattern, $subject);
    if ($result === false) {
        throw new \RuntimeException('invalid regex');
    }
    return $result;
}
===expect===
