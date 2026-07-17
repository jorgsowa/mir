===description===
`@return B<static>` on a template-free method: the outer class name is
namespace-qualified at collection time (previously left relative unless the
method declared its own @template), and the nested `static` resolves to the
called subclass, so chained calls keep their template bindings instead of
degrading to mixed.
===config===
suppress=UnusedVariable
===file===
<?php
namespace NS;

/** @template T */
class B {
    /** @return T|null */
    public function first() { return null; }
}

class M {
    /** @return B<static> */
    public function q(): B { return new B(); }
}

class C extends M {}

$c = new C();
$q = $c->q();
/** @mir-check $q is NS\B<NS\C> */
$first = $q->first();
/** @mir-check $first is NS\C|null */
===expect===
