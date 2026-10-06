===description===
Every named @var tag in a docblock applies, not only the last one.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo { public function m(): int { return 1; } }
class Bar { public function n(): int { return 2; } }

function mk(): mixed { return null; }

function pair(): int {
    /**
     * @var Foo $a
     * @var Bar $b
     */
    [$a, $b] = [mk(), mk()];
    /**
     * @mir-check $a is Foo
     * @mir-check $b is Bar
     */
    return $a->m() + $b->n();
}

function static_vars(): void {
    /**
     * @var int    $n
     * @var string $s
     */
    static $n = 0, $s = '';
    /** @mir-check $n is int */
    echo $n;
    /** @mir-check $s is string */
    echo $s;
}

function mixed_named_and_other(): int {
    $x = mk();
    $y = mk();
    /**
     * @var Foo $x
     * @var Bar $y
     * @mir-check $x is Foo
     * @mir-check $y is Bar
     */
    return $x->m() + $y->n();
}
