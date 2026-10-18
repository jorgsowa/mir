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
$b = [] |> (new Text())->length(...);
$c = "str" |> (fn(int $n): int => $n);
$d = 1 |> double(...) |> strtoupper(...);
===expect===
InvalidArgument@7:5-7:10: Argument $x of double() expects 'int', got '"str"'
InvalidArgument@8:5-8:7: Argument $s of length() expects 'string', got 'array{}'
InvalidArgument@9:5-9:10: Argument $n of {closure}() expects 'int', got '"str"'
ArgumentTypeCoercion@10:5-10:21: Argument $string of strtoupper() expects 'string', got 'int' — coercion may fail at runtime
