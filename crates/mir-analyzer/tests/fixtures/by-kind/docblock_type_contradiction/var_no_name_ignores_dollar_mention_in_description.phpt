===description===
A nameless @var's free-form description can mention an unrelated $variable
in passing — that mention must not be mistaken for the tag's own name (which
would silently stop the bare-form annotation from applying to the real LHS
below it, since `apply_post_narrow`'s named-form only applies on an exact
name match).
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
function compute(): \stdClass {
    return new \stdClass();
}

/** @var Foo This is a description mentioning $other in passing */
$x = compute();
/** @mir-check $x is Foo */
$_ = $x;
===expect===
