===description===
Reference reuse declared in else
===config===
suppress=UnusedVariable
===file===
<?php
/** @var array<int> */
$arr = [];

if (!isset($arr[0])) {
} else {
    $var = &$arr[0];
    $var += 1;
}

$var = "foo";

===expect===
UnsupportedReferenceUsage@7:5-7:20: Reference assignment is not supported
