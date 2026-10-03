===description===
intdiv() with a literal-zero second argument throws the exact same
DivisionByZeroError as `$a / 0` — the operator check already caught the
binary form, but intdiv() only narrowed its return type, never reporting
the same unconditional runtime error for a definite-zero divisor.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function intdiv_by_zero(int $x): int {
    return intdiv($x, 0);
//                    ^ DivisionByZero: Division by zero: right operand of 'intdiv' is always 0
}

function intdiv_by_nonzero(int $x): int {
    return intdiv($x, 5);
}
===expect===
