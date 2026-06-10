===description===
Reference reuse foreach value
===ignore===
TODO
===file===
<?php
/** @var array<int> */
$arr = [];

foreach ($arr as &$var) {
    $var += 1;
}

$var = "foo";

===expect===
