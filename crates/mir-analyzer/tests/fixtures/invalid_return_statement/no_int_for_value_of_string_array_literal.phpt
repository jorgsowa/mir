===description===
noIntForValueOfStringArrayLiteral
===file===
<?php
class A {
    /**
     * @return value-of<array<int, string>>
     */
    public function getValue() {
        return 42;
    }
}

===expect===
InvalidReturnStatement
===ignore===
TODO
