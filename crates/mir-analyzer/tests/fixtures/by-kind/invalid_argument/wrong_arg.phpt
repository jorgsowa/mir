===description===
Wrong arg
===ignore===
TODO
===file===
<?php
$bar = ["foo", "bar"];

$bam = array_map(
    function(int $a): int {
        return $a + 1;
    },
    $bar
);
===expect===
