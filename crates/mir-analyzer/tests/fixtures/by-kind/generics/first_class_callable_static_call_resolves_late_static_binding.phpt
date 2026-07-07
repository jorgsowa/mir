===description===
FN: `Child::create(...)` (first-class callable syntax on a static call) built
its closure's return type from the raw, unsubstituted `@return static`,
never resolving `static`/`self` to the receiver's concrete class the way a
direct `Child::create()` call already does via substitute_static_in_return
— so invoking the resulting closure returned the wrong (declaring) class.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {
    /** @return static */
    public static function create(): static {
        return new static();
    }
}
class Child extends Base {}

$fn = Child::create(...);
$obj = $fn();
/** @mir-check $obj is Child */
echo "ok";
===expect===
