===description===
`%` casts both operands to int, so its result is `int` for numeric-string and
float operands too, and passes to an int parameter without a float cast.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<int, string> $map */
function fromString(string $number, array $map): string {
    return $map[$number % 23] ?? '';
}

function fromFloat(float $f, int $i): void {
    $a = $f % 2;
    /** @mir-check $a is int */
    $b = $i % $f;
    /** @mir-check $b is int */
    $c = $i;
    $c %= 1.5;
    /** @mir-check $c is int */
    echo $a + $b + $c;
}

function takesInt(int $n): void { echo $n; }
takesInt(((string) rand()) % 7);
