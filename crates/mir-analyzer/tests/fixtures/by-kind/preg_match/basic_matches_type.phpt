===description===
preg_match without PREG_OFFSET_CAPTURE writes list<string> to $matches.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function run(string $s): void {
    preg_match('/(\d+)/', $s, $matches);
    /** @mir-check $matches is list<string> */
    $_ = $matches;
}
===expect===
