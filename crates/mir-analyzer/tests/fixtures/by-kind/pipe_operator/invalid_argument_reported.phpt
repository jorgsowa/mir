===description===
The left operand of `|>` is checked against the callable's parameter type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function double(int $x): int { return $x * 2; }
final class Text {
    public function length(string $s): int { return strlen($s); }
}

$a = "str" |> double(...);
//   ^^^^^ InvalidArgument: Argument $x of double() expects 'int', got '"str"'
$b = [] |> (new Text())->length(...);
//   ^^ InvalidArgument: Argument $s of length() expects 'string', got 'array{}'
$c = "str" |> (fn(int $n): int => $n);
//   ^^^^^ InvalidArgument: Argument $n of {closure}() expects 'int', got '"str"'
$d = 1 |> double(...) |> strtoupper(...);
//   ^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $string of strtoupper() expects 'string', got 'int' — coercion may fail at runtime
