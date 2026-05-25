===description===
Only defined values of constant list
===file===
<?php
class A {
    const FOO = [
        "bar"
    ];
    /** @return key-of<A::FOO> */
    public function getValue() {
        return "adams";
    }
}

===expect===
InvalidReturnStatement
===ignore===
TODO
