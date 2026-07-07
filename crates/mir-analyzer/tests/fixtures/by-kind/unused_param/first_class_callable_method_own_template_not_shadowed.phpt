===description===
FP: a method-level `@template T` shadowing a same-named class-level
template must stay unbound in a first-class-callable closure, the same way
the direct-call path (method.rs) deliberately keeps it unbound so argument
checking doesn't apply the class's binding to the wrong template. The FCC
path substituted the class-level binding (Box<string>'s T -> string)
unfiltered, baking it into a parameter that should have stayed generic —
producing a false-positive InvalidArgument that the direct call correctly
avoids.
===config===
suppress=UnusedVariable,UnusedParam,ShadowedTemplateParam
===file===
<?php
/** @template T */
class Box {
    /** @param T $seed */
    public function __construct($seed) {}

    /**
     * @template T
     * @param T $value
     * @return T
     */
    public function transform($value) {
        return $value;
    }
}

$box = new Box('hello');
$box->transform(42);

$fn = $box->transform(...);
$fn(42);
===expect===
