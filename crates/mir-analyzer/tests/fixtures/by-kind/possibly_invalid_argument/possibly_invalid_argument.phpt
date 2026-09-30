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
//  ^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $array of array_map() expects 'array', possibly different type '"a"|array{0: "b"}' provided
);
===expect===
