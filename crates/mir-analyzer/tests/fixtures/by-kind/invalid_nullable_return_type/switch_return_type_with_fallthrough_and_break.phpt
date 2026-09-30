===description===
Switch return type with fallthrough and break
===file===
<?php
class A {
    /** @return bool */
    public function fooFoo() {
//                           ^ +7:5 InvalidReturnType: Return type 'void' is not compatible with declared 'bool'
        switch (rand(0,10)) {
            case 1:
                break;
            default:
                return true;
        }
    }
}
===expect===
