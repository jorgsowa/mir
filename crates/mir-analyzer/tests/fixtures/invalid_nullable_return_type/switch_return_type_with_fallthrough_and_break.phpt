===description===
Switch return type with fallthrough and break
===file===
<?php
class A {
    /** @return bool */
    public function fooFoo() {
        switch (rand(0,10)) {
            case 1:
                break;
            default:
                return true;
        }
    }
}
===expect===
InvalidNullableReturnType
===ignore===
TODO
