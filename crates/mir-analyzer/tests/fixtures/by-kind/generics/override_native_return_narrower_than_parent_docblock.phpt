===description===
A narrower native return type on an override is not replaced by the parent's docblock
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
abstract class Base {
    /** @return Base<T> */
    abstract public function self(): Base;
}

/**
 * @template T
 * @extends Base<T>
 */
final class Child extends Base {
    public function self(): Child { return $this; }
}

function sink(mixed $x): void {}

/** @param Child<int> $c */
function run(Child $c): void {
    $r = $c->self();
    /** @mir-check $r is Child */
    sink($r);
}
===expect===
