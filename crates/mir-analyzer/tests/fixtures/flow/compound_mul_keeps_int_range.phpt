===description===
`*=` keeps the int range like `+=`/`-=`, so a range-typed property or return
accepts the result and literal operands fold.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Delay {
    /** @var positive-int */
    public int $seconds = 1;

    public function grow(): void {
        $next = $this->seconds;
        $next *= 2;
        $this->seconds = $next;
    }
}

/** @return positive-int */
function double(): int {
    $n = 4;
    $n *= 2;
    return $n;
}

function literals(): void {
    $m = 3;
    $m *= 2;
    /** @mir-check $m is 6 */
    $_ = $m;
}

/** @param positive-int $n */
function ranged(int $n): void {
    $n *= 3;
    /** @mir-check $n is int<3, max> */
    $_ = $n;
}

function unknownOperand(int $a, int $b): void {
    $a *= $b;
    /** @mir-check $a is int */
    $_ = $a;
}

function floatOperand(): void {
    $x = 2;
    $x *= 1.5;
    /** @mir-check $x is float */
    $_ = $x;
}
