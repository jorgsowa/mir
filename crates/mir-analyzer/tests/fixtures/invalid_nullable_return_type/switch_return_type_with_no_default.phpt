===description===
Switch return type with no default
===file===
<?php
class A {
    /** @return bool */
    public function fooFoo() {
        switch (rand(0,10)) {
            case 1:
            case 2:
                return true;
        }
    }
}
===expect===
InvalidNullableReturnType
===ignore===
TODO
