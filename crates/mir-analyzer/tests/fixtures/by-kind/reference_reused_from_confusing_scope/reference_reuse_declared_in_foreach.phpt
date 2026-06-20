===description===
Reference reuse declared in foreach
===config===
suppress=UnusedVariable
===file===
<?php
/** @var array<int> */
$arr = [];

foreach ($arr as $val) {
    $var = &$val;
    $var += 1;
}

$var = "foo";

===expect===
