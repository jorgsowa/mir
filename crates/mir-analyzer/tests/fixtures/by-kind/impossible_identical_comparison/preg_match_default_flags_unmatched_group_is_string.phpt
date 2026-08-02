===description===
Without PREG_UNMATCHED_AS_NULL, an unmatched capture group is always the empty
string, never null — comparing it to null must still be flagged.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function parseNumber(string $value): void {
    preg_match('/(?P<integral>\d+)(\.(?P<fraction>\d+))?/', $value, $matches);

    if ($matches['fraction'] === null) {
        echo "no fraction\n";
    }
}
===expect===
ImpossibleIdenticalComparison@6:8-6:37: '===' between 'string' and 'null' is always false — these types can never be identical
