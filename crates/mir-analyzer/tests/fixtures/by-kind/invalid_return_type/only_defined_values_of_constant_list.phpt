===description===
Only defined values of constant list
===ignore===
TODO
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
