===description===
Warn about mismatching interface param doc
===file===
<?php
class A {}
class B {}

interface X {
    /**
     * @param B $class
     */
    public function boo(A $class): void {}
}
===expect===
MismatchingDocblockParamType
===ignore===
TODO
