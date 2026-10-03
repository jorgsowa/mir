===description===
gettype match arm with invalid type string
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = rand(0, 10) ? 1 : "two";

$x = match (gettype($a)) {
    "int" => 1,
//  ^^^^^ UnevaluatedCode: Unevaluated code: gettype() never returns "int" (did you mean "integer"?)
    "integer", "string" => 2,
    default => 3,
};
===expect===
