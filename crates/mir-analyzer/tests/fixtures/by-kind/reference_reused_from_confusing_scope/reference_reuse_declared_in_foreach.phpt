===description===
Reference reuse declared in foreach
===ignore===
TODO
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
