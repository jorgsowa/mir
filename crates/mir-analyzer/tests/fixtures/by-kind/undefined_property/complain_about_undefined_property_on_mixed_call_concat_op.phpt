===description===
Complain about undefined property on mixed call concat op
===file===
<?php
class A {
    /**
     * @suppress MixedMethodCall
//               ^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'MixedMethodCall' is never used
     */
    public function foo(object $a) : void {
        $a->bar("bat" . $this->baz);
//                             ^^^ UndefinedProperty: Property A::$baz does not exist
    }
}
===expect===
