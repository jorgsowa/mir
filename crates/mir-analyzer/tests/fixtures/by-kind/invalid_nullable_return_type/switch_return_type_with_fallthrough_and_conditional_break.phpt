===description===
Switch return type with fallthrough and conditional break
===file===
<?php
class A {
    /** @return bool */
    public function fooFoo() {
        switch (rand(0,10)) {
            case 1:
                if (rand(0,10) === 5) {
                    break;
                }
            default:
                return true;
        }
    }
}
===expect===
InvalidReturnType@4:30-13:31: Return type 'void' is not compatible with declared 'bool'
