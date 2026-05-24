===description===
unsetOnlyPreventsReferenceConfusionAfterCall
===file===
<?php
$arr = [1, 2, 3];
foreach ($arr as &$i) {
    ++$i;
}

for ($i = 0; $i < 10; ++$i) {
    echo $i;
}

unset($i);

===expect===
ReferenceReusedFromConfusingScope
===ignore===
TODO
