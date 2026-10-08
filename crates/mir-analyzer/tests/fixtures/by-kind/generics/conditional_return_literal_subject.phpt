===description===
`@return (T is 'a' ? A : B)` and `($p is 5 ? A : B)` pick a branch from a literal argument; a broader or ambiguous argument keeps both.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}

/**
 * @template T of string
 * @param T $t
 * @return (T is 'a' ? A : B)
 */
function byTemplate(string $t) { return new A(); }

/** @return ($kind is 'a' ? A : B) */
function byParam(string $kind) { return new A(); }

/** @return ($n is 5 ? A : B) */
function byInt(int|string $n) { return new A(); }

function test(string $s, string|int $mixedLit): void {
    $a = byTemplate('a');
    /** @mir-check $a is A */
    $b = byTemplate('b');
    /** @mir-check $b is B */
    $c = byParam('a');
    /** @mir-check $c is A */
    $d = byParam('z');
    /** @mir-check $d is B */
    $e = byInt(5);
    /** @mir-check $e is A */
    $f = byInt(6);
    /** @mir-check $f is B */
    $g = byInt('5');
    /** @mir-check $g is B */
    $h = byParam($s);
    /** @mir-check $h is A|B */
    $i = byParam(rand() ? 'a' : 'b');
    /** @mir-check $i is A|B */
    $j = byInt($mixedLit);
    /** @mir-check $j is A|B */
}
