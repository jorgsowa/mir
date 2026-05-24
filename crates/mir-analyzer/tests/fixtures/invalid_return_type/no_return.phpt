===description===
noReturn
===file===
<?php
$bar = ["foo", "bar"];

$bam = array_map(
    function(string $a): string {
    },
    $bar
);
===expect===
InvalidReturnType
===ignore===
TODO
