===description===
FN: `new IntBox(42)` correctly infers `Box<int>` from constructor args, but
`new IntBox(42)` (a plain subclass that doesn't redeclare `@template`) lost
all inference — `class_template_params(IntBox)` returned empty since
IntBox declares no template of its own, so infer_new_type_params bailed
out before ever binding T from the constructor argument.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;
    /** @param T $value */
    public function __construct($value) { $this->value = $value; }
    /** @return T */
    public function get() { return $this->value; }
}
class IntBox extends Box {}

$b = new IntBox(42);
$v = $b->get();
/** @mir-check $v is int */
$_ = $v;
===expect===
