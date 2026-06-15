===description===
Reference reuse declared in elseif
===config===
suppress=MissingThrowsDocblock,UnusedVariable
===file===
<?php
/** @var array<int> */
$arr = [];

if (random_int(0, 1)) {
} elseif (isset($arr[0])) {
    $var = &$arr[0];
    $var += 1;
}

$var = "foo";

===expect===
UnsupportedReferenceUsage@7:4-7:19: Reference assignment is not supported
