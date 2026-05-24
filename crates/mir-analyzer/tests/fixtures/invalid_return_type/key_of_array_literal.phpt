===description===
keyOfArrayLiteral
===file===
<?php
class A {
    /**
     * @return key-of<array<int, string>>
     */
    public function getKey() {
        return "foo";
    }
}

===expect===
InvalidReturnStatement
===ignore===
TODO
