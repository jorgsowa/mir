===description===
No return
===config===
suppress=UnusedVariable
===file===
<?php
$bar = ["foo", "bar"];

$bam = array_map(
    function(string $a): string {
    },
    $bar
);
===expect===
