===description===
A method's own `@template` (not the class's) must be substituted into its
own `@param-out` type on an instance call, the same way `static_call.rs`
already does for `Foo::method(...)`. The instance-call write-back loop used
`effective_params`, whose out_ty deliberately has the method's own template
name stripped out (so `check_args` can still infer it from the arguments),
so the method-level template leaked to the caller unsubstituted.
===config===
suppress=UnusedVariable
===file===
<?php
class Box2 {
    /**
     * @template U
     * @param U $val
     * @param-out U $out
     */
    public function copyInto($val, mixed &$out): void {
        $out = $val;
    }
}

$b = new Box2();
$b->copyInto(42, $result);
/** @mir-check $result is int */
$_ = $result;
===expect===
