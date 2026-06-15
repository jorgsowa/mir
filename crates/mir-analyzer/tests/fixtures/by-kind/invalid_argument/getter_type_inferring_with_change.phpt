===description===
Getter type inferring with change
===config===
suppress=MissingPropertyType
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
PossiblyInvalidArgument@16:16-16:30: Argument $string of strlen() expects 'string', possibly different type 'int|string|null' provided
PossiblyNullArgument@16:16-16:30: Argument $string of strlen() might be null
