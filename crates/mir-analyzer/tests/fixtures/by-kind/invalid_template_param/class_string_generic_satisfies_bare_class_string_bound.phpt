===description===
FP: `class-string<X>` was rejected by a bare `class-string` template bound
(InvalidTemplateParam) on `new`, function, method and static calls, though it
is strictly narrower than the bound.
===config===
<mir>
  <issueHandlers>
    <UnusedParameter errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class X {}

/** @template T of class-string */
class Box {
    /** @param T $c */
    public function __construct(public string $c) {}

    /**
     * @template U of class-string
     * @param U $c
     * @return U
     */
    public function m(string $c): string { return $c; }

    /**
     * @template U of class-string
     * @param U $c
     * @return U
     */
    public static function s(string $c): string { return $c; }
}

/**
 * @template T of class-string
 * @param T $c
 * @return T
 */
function ident(string $c): string { return $c; }

/** @param class-string<X> $c */
function test(string $c, string $plain): void {
    $b = new Box($c);
    /** @mir-check $b is Box<class-string<X>> */
    $r1 = ident($c);
    /** @mir-check $r1 is class-string<X> */
    $r2 = $b->m($c);
    /** @mir-check $r2 is class-string<X> */
    $r3 = Box::s($c);
    /** @mir-check $r3 is class-string<X> */
    ident($plain);
//  ^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'string' does not satisfy bound 'class-string'
}
===expect===
