===description===
switchReturnTypeWithFallthroughAndConditionalBreak
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
InvalidNullableReturnType
===ignore===
TODO
