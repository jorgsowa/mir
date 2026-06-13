===description===
Warn about mismatching class param doc
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B {}

class X {
    /**
     * @param B $class
     */
    public function boo(A $class): void {}
}
===expect===
