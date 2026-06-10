===description===
Enforce parameter inheritance with inherit doc
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
     * @inheritdoc
     */
    public function boo(A $class): void {}
}

(new Y())->boo(new A());
===expect===
