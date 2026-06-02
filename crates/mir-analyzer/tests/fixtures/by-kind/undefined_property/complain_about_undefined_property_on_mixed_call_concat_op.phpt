===description===
Complain about undefined property on mixed call concat op
===file===
<?php
class A {
    /**
     * @suppress MixedMethodCall
     */
    public function foo(object $a) : void {
        $a->bar("bat" . $this->baz);
    }
}
===expect===
UndefinedProperty@7:32-7:35: Property A::$baz does not exist
