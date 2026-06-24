===description===
@phpstan-param-out is an alias for @param-out and must be recognized.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @phpstan-param-out list<string> $out
 */
function collectNames(mixed &$out): void {
    $out = ["Alice", "Bob"];
}

$names = null;
collectNames($names);
/** @mir-check $names is list<string> */
$_ = $names;
===expect===
