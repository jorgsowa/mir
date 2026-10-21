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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: Interface function X::boo() cannot contain body
}
