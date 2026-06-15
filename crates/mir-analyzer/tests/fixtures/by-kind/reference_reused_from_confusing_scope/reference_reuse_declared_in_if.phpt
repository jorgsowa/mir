===description===
Reference reuse declared in if
===config===
suppress=UnusedVariable
===file===
<?php
/** @var array<int> */
$arr = [];

if (isset($arr[0])) {
    $var = &$arr[0];
    $var += 1;
}

$var = "foo";

===expect===
UnsupportedReferenceUsage@6:4-6:19: Reference assignment is not supported
