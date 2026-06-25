===description===
FP: preg_match with PREG_OFFSET_CAPTURE — accessing $matches[0][0] (string) and
$matches[0][1] (int) must not emit NonExistentArrayOffset or InvalidArgument.
Before the fix, $matches was typed as list<string> so $matches[0][1] triggered
NonExistentArrayOffset because string has no index 1.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function extractOffset(string $s): int {
    if (preg_match('/(\d+)/', $s, $matches, PREG_OFFSET_CAPTURE)) {
        $fullMatch = $matches[0][0]; // matched text — string
        $offset    = $matches[0][1]; // byte offset — int
        return $offset;
    }
    return -1;
}
===expect===
