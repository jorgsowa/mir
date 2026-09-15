===description===
Counter increments inside loops carry state into the next iteration and must not
be reported as unused solely because the final iteration has no successor.
===file===
<?php

/** @param list<string> $lines */
function findPostfix(array $lines, string $needle): int {
    $offset = 1;
    foreach ($lines as $line) {
        if ($line === $needle) {
            return $offset;
        }
        $offset++;
        /** @mir-check $offset is int */
    }
    return 0;
}

/** @param list<string> $lines */
function findPrefix(array $lines, string $needle): int {
    $offset = 1;
    foreach ($lines as $line) {
        ++$offset;
        if ($line === $needle) {
            return $offset;
        }
        /** @mir-check $offset is int */
    }
    return 0;
}
===expect===
