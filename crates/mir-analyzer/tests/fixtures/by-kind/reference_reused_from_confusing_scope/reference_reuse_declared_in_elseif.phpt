===description===
Reference reuse declared in elseif
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
UnsupportedReferenceUsage@7:5-7:20: Reference assignment is not supported
