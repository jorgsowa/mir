===description===
FP-P11: preg_match with PREG_UNMATCHED_AS_NULL. An unmatched named capture
group is reported as null (not "") when this flag is set, so comparing it
to null must not emit ImpossibleIdenticalComparison.
===config===
suppress=UnusedVariable,UnusedFunction,MixedArgument
php_version=8.2
===file===
<?php

function parseNumber(string $value): void {
    preg_match('/(?P<integral>\d+)(\.(?P<fraction>\d+))?/', $value, $matches, PREG_UNMATCHED_AS_NULL);

    if ($matches['fraction'] === null) {
        echo "no fraction\n";
    }
}
===expect===
