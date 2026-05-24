===description===
possiblyInvalidArgument
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
PossiblyInvalidArgument
===ignore===
TODO
