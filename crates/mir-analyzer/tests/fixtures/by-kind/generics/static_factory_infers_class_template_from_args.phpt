===description===
FN: a static factory method (`@return static`) on a generic class never
bound the CLASS's own `@template T` from the call's arguments — only the
method's own separately-declared templates were inferred. `Box::make(42)`
resolved to a bare, unparameterized `Box` instead of `Box<int>`, because
`static`'s receiver type params were substituted before the class-level
template had any chance to be inferred from the arguments (unlike `new
Box(42)`, which already infers class templates from constructor args).
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

    /**
     * @param T $value
     * @return static
     */
    public static function make($value): static {
        return new static($value);
    }
}

$box = Box::make(42);
/** @mir-check $box is Box<int> */
echo "ok";
===expect===
