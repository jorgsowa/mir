===description===
Reference reuse declared in if
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
UnsupportedReferenceUsage@6:5-6:20: Reference assignment is not supported
