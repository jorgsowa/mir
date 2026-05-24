===description===
referenceReuseDeeplyNested
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
ReferenceReusedFromConfusingScope
===ignore===
TODO
