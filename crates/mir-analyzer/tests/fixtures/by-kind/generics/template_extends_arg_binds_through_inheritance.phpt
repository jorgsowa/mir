===description===
`@template-extends Base<U>` type args referencing the subclass's own template
param must be stored as template params, not namespace-qualified into a
phantom `NS\U` class — otherwise a `Base` method's `@return T` can't chase
T -> U -> the receiver's concrete binding and leaks the raw template atom.
===config===
suppress=UnusedVariable
===file===
<?php
namespace NS;

/** @template T */
class Base {
    /** @return T|null */
    public function first() { return null; }
}

/**
 * @template U
 * @template-extends Base<U>
 */
class Child extends Base {}

class Thing {}

class Holder {
    /** @return Child<Thing> */
    public function child(): Child { return new Child(); }
}

$h = new Holder();
$c = $h->child();
/** @mir-check $c is NS\Child<NS\Thing> */
$x = $c->first();
/** @mir-check $x is NS\Thing|null */
===expect===
