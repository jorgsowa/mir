===description===
FN: a property declared `@var T` resolved to the bare, unresolved template
atom T instead of the receiver's actual concrete type param — e.g. `new
Box(42)` on `class Box { /** @var T */ public $value; }` typed `$box->value`
as `T`, not `int`. Property-type resolution never substituted the
receiver's own `type_params` (`Box<int>`'s `T -> int`) into the declared
property type at all.
===config===
suppress=MissingPropertyType,UnusedVariable
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }
}

$box = new Box(42);
$v = $box->value;
/** @mir-check $v is int */
echo "ok";
===expect===
