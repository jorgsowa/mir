===description===
Multiple @param-out annotations on different parameters — each variable should
receive its declared out-type after the call.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param-out string $name
 * @param-out int $age
 */
function unpack(array $data, mixed &$name, mixed &$age): void {
    $name = $data["name"];
    $age = $data["age"];
}

unpack(["name" => "Alice", "age" => 30], $n, $a);
/** @mir-check $n is string */
$_ = $n;
/** @mir-check $a is int */
$_ = $a;
===expect===
