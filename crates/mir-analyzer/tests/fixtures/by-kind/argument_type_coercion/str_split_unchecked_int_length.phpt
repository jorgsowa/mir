===description===
A plain `int` length may be below 1, which `str_split()` and `mb_str_split()` reject.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function chunks(string $value, int $size): void {
    str_split($value, $size);
//                    ^^^^^ ArgumentTypeCoercion: Argument $length of str_split() expects 'positive-int', got 'int' — coercion may fail at runtime
    mb_str_split($value, $size);
//                       ^^^^^ ArgumentTypeCoercion: Argument $length of mb_str_split() expects 'positive-int', got 'int' — coercion may fail at runtime
}
