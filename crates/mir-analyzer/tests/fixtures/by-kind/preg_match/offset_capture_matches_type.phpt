===description===
preg_match with PREG_OFFSET_CAPTURE writes list<array{0: string, 1: int}> to $matches.
Each entry is a [matched_text, byte_offset] pair.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function run(string $s): void {
    preg_match('/(\d+)/', $s, $matches, PREG_OFFSET_CAPTURE);
    /** @mir-check $matches is list<array{0: string, 1: int}> */
    $_ = $matches;
}
===expect===
