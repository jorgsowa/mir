===description===
Getter type inferring with change
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
PossiblyNullArgument@16:17-16:31: Argument $string of strlen() might be null
InvalidArgument@16:17-16:31: Argument $string of strlen() expects 'string', got 'int|string|null'
