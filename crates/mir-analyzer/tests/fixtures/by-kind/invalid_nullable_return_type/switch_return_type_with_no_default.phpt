===description===
Switch return type with no default
===file===
<?php
class A {
    /** @return bool */
    public function fooFoo() {
//                           ^ +6:5 InvalidReturnType: Return type 'void' is not compatible with declared 'bool'
        switch (rand(0,10)) {
            case 1:
            case 2:
                return true;
        }
    }
}
===expect===
