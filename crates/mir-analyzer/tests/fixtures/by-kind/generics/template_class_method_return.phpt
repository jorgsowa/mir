===description===
G1: a generic class Box<T> with a get() method returns T as the concrete type when the
class was constructed with a known argument, so @mir-check can verify the result type.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @param T $value */
    public function __construct(private $value) {}

    /** @return T */
    public function get() { return $this->value; }
}

$strBox = new Box("hello");
$v1 = $strBox->get();
/** @mir-check $v1 is string */

$intBox = new Box(42);
$v2 = $intBox->get();
/** @mir-check $v2 is 42 */
===expect===
