===description===
Reference reuse declared in for
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
