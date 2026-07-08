===description===
FN: same root cause as new-on-subclass — `Box::make(42)` already inferred
`Box<int>` from args, but calling the same inherited static factory
through a subclass (`IntBox::make(42)`) lost inference because the
class-template lookup used the LSB-resolved call class (IntBox, no own
template) instead of walking up to Box, where the template is declared.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @param T $value */
    public function __construct(private $value) {}
    /** @param T $value @return static */
    public static function make($value): static { return new static($value); }
    /** @return T */
    public function get() { return $this->value; }
}
class IntBox extends Box {}

$b = IntBox::make(42);
$v = $b->get();
/** @mir-check $v is int */
$_ = $v;
===expect===
