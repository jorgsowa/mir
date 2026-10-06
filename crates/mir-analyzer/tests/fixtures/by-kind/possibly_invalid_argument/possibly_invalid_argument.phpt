===description===
Possibly invalid argument
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
