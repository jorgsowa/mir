===description===
getterTypeInferringWithChange
===file===
<?php
class A {
    /** @var int|string|null */
    public $val;

    /** @return int|string|null */
    final public function getValue() {
        return $this->val;
    }
}

$a = new A();

if (is_string($a->getValue())) {
    $a->val = 5;
    echo strlen($a->getValue());
}
===expect===
InvalidScalarArgument
===ignore===
TODO
