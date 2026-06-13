===description===
Reference reuse foreach value
===config===
suppress=UnusedForeachValue
===file===
<?php
/** @var array<int> */
$arr = [];

foreach ($arr as &$var) {
    $var += 1;
}

$var = "foo";

===expect===
