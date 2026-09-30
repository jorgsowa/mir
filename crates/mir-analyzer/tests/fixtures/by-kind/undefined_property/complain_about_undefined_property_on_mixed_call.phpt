===description===
Complain about undefined property on mixed call
===file===
<?php
class C {
    /** @param mixed $a */
    public function foo($a) : void {
        /** @suppress MixedMethodCall */
        $a->bar($this->d);
//                     ^ UndefinedProperty: Property C::$d does not exist
    }
}
===expect===
