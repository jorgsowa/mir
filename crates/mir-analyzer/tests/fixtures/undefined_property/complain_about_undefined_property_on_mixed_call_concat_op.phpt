===description===
complainAboutUndefinedPropertyOnMixedCallConcatOp
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
