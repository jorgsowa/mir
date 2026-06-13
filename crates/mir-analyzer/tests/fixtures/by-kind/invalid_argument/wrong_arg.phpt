===description===
Wrong arg
===config===
suppress=UnusedVariable
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
