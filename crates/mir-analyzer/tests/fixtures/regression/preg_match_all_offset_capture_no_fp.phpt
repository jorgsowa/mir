===description===
FP-D(a): preg_match_all with PREG_OFFSET_CAPTURE. Accessing $matches[0][0][1] (the
byte offset) must not emit NonExistentArrayOffset.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function parseAllOffsets(string $input): array {
    preg_match_all('/\d+/', $input, $matches, PREG_OFFSET_CAPTURE);
    $result = [];
    foreach ($matches[0] as $match) {
        $result[] = $match[1]; // byte offset — valid int access
    }
    return $result;
}
===expect===
