===description===
preg_match_all without PREG_OFFSET_CAPTURE writes list<list<string>> to $matches.
Default flag PREG_PATTERN_ORDER: $matches[0] is all full matches, $matches[1] all
captures of group 1, etc. — each is list<string>.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function run(string $s): void {
    preg_match_all('/(\d+)/', $s, $matches);
    /** @mir-check $matches is list<list<string>> */
    $_ = $matches;
}
===expect===
