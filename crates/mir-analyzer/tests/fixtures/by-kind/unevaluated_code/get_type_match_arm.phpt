===description===
gettype match arm with invalid type string
===config===
suppress=UnusedVariable
===file===
<?php
$a = rand(0, 10) ? 1 : "two";

$x = match (gettype($a)) {
    "int" => 1,
    "integer", "string" => 2,
    default => 3,
};
===expect===
UnevaluatedCode@5:4-5:9: Unevaluated code: gettype() never returns "int" (did you mean "integer"?)
