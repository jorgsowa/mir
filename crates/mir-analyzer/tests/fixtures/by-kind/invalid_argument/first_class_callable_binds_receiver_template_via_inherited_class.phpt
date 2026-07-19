===description===
FN: first-class-callable syntax used class_template_params (own-declared
only) to look up the receiver's template slots, so a subclass that inherits
a template without redeclaring its own `@template` (`class IntBox extends
Box {}`) lost the substitution entirely — the direct-call form already used
effective_class_template_params for this. Covers both instance-method FCC
(`$box->set(...)`) and the `$box::method(...)` static-call-through-a-typed-
receiver form (dynamic class-expr path, the one shape that actually carries
receiver type params into a static call).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @template T */
class Box {
    /** @param T $x */
    public function set($x): void {}

    /** @param T $x */
    public static function make($x): void {}
}

class IntBox extends Box {}

/** @param IntBox<int> $box */
function viaInstanceFcc(IntBox $box): void {
    $fn = $box->set(...);
    $fn("bad-not-int");
}

/** @param IntBox<int> $box */
function viaStaticFccThroughReceiver(IntBox $box): void {
    $fn = $box::make(...);
    $fn("also-bad");
}
===expect===
InvalidArgument@16:8-16:21: Argument $x of {closure}() expects 'int', got '"bad-not-int"'
InvalidArgument@22:8-22:18: Argument $x of {closure}() expects 'int', got '"also-bad"'
