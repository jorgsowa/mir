===description===
referenceReuseDeclaredInElseif
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
ReferenceReusedFromConfusingScope
===ignore===
TODO
