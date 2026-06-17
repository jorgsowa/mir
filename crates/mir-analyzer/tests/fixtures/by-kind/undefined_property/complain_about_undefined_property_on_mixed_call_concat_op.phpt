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
UnusedSuppress@6:0-6:0: Suppress annotation for 'MixedMethodCall' is never used
UndefinedProperty@7:31-7:34: Property A::$baz does not exist
