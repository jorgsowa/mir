===description===
The result of a pipe is the call's return value, so passing it on or using it
as an operand is not reported as a `Closure` or string mismatch.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function double(int $x): int { return $x * 2; }
function takesInt(int $i): void {}
function takesString(string $s): void {}

takesInt(5 |> double(...));
takesInt([1, 2] |> (fn(array $x): int => count($x)));
takesInt(5 |> 'double');
takesString("hi" |> strtoupper(...));
echo (5 |> double(...)) + 1;
echo strlen("hi" |> strtoupper(...));

function fromPipe(int $x): int {
    return $x |> double(...);
}
