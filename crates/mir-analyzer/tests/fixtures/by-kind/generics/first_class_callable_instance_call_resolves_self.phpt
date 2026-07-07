===description===
FN: `$obj->makeSelf(...)` (first-class callable syntax on an instance call)
built its closure's return type from the raw, unsubstituted `@return
static`, the same gap as the static-call-syntax case, via the Method/
NullsafeMethod arm of build_closure_from_resolved_params.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {
    /** @return static */
    public function makeSelf(): static {
        return new static();
    }
}
class Child extends Base {}

$child = new Child();
$fn = $child->makeSelf(...);
$obj = $fn();
/** @mir-check $obj is Child */
echo "ok";
===expect===
