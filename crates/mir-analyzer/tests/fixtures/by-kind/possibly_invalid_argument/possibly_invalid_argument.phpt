===description===
Possibly invalid argument
===config===
suppress=UnusedVariable
===file===
<?php
$foo = [
    "a",
    ["b"],
];

$a = array_map(
    function (string $uuid): string {
        return $uuid;
    },
    $foo[rand(0, 1)]
);
===expect===
PossiblyInvalidArgument@11:4-11:20: Argument $array of array_map() expects 'array<mixed, mixed>', possibly different type '"a"|array{0: "b"}' provided
