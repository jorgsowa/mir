===description===
enforceParameterInheritanceWithCapitalizedInheritDoc
===file===
<?php
class A {}
class B extends A {}

class X {
    /**
     * @param B $class
     */
    public function boo(A $class): void {}
}

class Y extends X {
    /**
     * @inheritDoc
     */
    public function boo(A $class): void {}
}

(new Y())->boo(new A());
===expect===
ArgumentTypeCoercion
===ignore===
TODO
