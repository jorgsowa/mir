===description===
Void returning array map
===file===
<?php
$arr = array_map(
    function(int $i) : void {
        echo $i;
    },
    [1, 2, 3]
);

foreach ($arr as $a) {
    if ($a) {}
}
===expect===
