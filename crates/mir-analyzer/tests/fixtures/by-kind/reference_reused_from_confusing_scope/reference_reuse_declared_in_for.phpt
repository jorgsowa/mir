===description===
Reference reuse declared in for
===config===
suppress=UnusedVariable
===file===
<?php
/** @var list<int> */
$arr = [];

for ($i = 0; $i < count($arr); ++$i) {
    $var = &$arr[$i];
    $var += 1;
}

$var = "foo";

===expect===
