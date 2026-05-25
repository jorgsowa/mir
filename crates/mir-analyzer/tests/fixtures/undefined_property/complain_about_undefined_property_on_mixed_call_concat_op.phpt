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
UndefinedThisPropertyFetch
===ignore===
TODO
