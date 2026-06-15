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
ParseError@9:4-9:42: Parse error: interface method cannot contain a body
