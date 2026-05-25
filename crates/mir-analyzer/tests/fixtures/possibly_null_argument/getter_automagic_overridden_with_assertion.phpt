===description===
Getter automagic overridden with assertion
===file===
<?php
class A {
    /** @var string|null */
    public $a;

    /** @assert-if-true string $this->a */
    function hasA() {
        return is_string($this->a);
    }

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
    if ($a->hasA()) {
        echo strlen($a->getA());
    }
}

foo(new AChild());
===expect===
PossiblyNullArgument
===ignore===
TODO
