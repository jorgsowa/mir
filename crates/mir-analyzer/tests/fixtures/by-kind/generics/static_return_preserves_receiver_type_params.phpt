===description===
G1: a fluent `: static`-returning method must preserve the receiver's own
inferred type params instead of erasing them to a bare class type — chaining
another generic-returning call on the result must still resolve correctly.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    private $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }

    /** @param T $value */
    public function withValue($value): static {
        $this->value = $value;
        return $this;
    }

    /** @return T */
    public function get() {
        return $this->value;
    }
}

$box = new Box(42);
$box2 = $box->withValue(43);
$v = $box2->get();
/** @mir-check $v is int */
===expect===
