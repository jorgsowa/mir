===description===
inheritMultipleInterfacesWithConflictingDocblocks
===file===
<?php
interface I1 {
    /** @return string */
    public function foo();
}
interface I2 {
    /** @return int */
    public function foo();
}
class A implements I1, I2 {
    public function foo() {
        return "hello";
    }
}
===expect===
InvalidReturnType
===ignore===
TODO
