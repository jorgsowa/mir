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
//                             ^^^ UndefinedProperty: Property A::$baz does not exist
    }
}
===expect===
UnusedSuppress@4:17-4:32: Suppress annotation for 'MixedMethodCall' is never used
