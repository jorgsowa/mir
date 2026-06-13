===description===
Reference reuse deeply nested
===config===
suppress=UnusedVariable
===file===
<?php
/** @var list<list<list<int>>> */
$arr = [];

for ($i = 0; $i < count($arr); ++$i) {
    foreach ($arr[$i] as $inner_arr) {
        if (isset($inner_arr[0])) {
            $var = &$inner_arr[0];
            $var += 1;
        }
    }
}

$var = "foo";

===expect===
UnsupportedReferenceUsage@8:13-8:34: Reference assignment is not supported
