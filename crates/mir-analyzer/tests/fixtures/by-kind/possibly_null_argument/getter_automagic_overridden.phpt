===description===
Getter automagic overridden
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string|null */
    public $a;

    /** @return string|null */
    function getA() {
        return $this->a;
    }
}

class AChild extends A {
    function getA() {
        return rand(0, 1) ? $this->a : null;
    }
}

function foo(A $a) : void {
    if ($a->getA() !== null) {
        echo strlen($a->getA());
    }
}

foo(new AChild());
===expect===
PossiblyNullArgument@20:20-20:30: Argument $string of strlen() might be null
