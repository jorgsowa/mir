===description===
In weak mode a numeric-string (typed, literal, or an int cast to string) is accepted by float and int|float params
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takes_number(int|float $x): void {}
function takes_float(float $x): void {}
function takes_int(int $x): void {}

/** @param numeric-string $ns */
function typed(string $s, $ns): void {
    takes_number($ns);
    takes_float($ns);
    takes_number("12");
    takes_float("1.5");
    takes_number(" 7");
    takes_number("1e3");
}

function cast_int(int $n, int|string $u): void {
    $s = (string)($n * 100);
    /** @mir-check $s is numeric-string */
    takes_number($s);
    echo ceil((string)$n);
    $t = (string)$u;
    /** @mir-check $t is string */
}
