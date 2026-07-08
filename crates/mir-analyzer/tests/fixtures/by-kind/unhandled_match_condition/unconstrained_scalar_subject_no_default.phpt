===description===
A match() on a plain (non-literal, non-enum) scalar subject with no default
arm can throw UnhandledMatchError for any value the arms don't happen to
list — check_match_exhaustiveness previously only proved exhaustiveness for
a finite literal-union or enum subject, silently accepting everything else.
===config===
suppress=UnusedParam
===file===
<?php
function no_default(int $x): string {
    return match ($x) { 1 => 'one', 2 => 'two' };
}

function with_default(int $x): string {
    return match ($x) { 1 => 'one', 2 => 'two', default => 'other' };
}
===expect===
UnhandledMatchCondition@3:11-3:48: Unhandled match condition: possibly-unmatched value of type 'int'
