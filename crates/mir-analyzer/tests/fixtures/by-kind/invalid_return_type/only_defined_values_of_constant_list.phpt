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
InvalidReturnType@8:8-8:23: Return type '"adams"' is not compatible with declared 'key-of<A::FOO>'
