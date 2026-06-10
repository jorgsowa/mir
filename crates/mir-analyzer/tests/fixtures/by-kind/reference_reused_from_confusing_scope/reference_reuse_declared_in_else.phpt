===description===
Reference reuse declared in else
===ignore===
TODO
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
