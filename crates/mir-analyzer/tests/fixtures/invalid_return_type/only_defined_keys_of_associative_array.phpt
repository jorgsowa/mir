===description===
onlyDefinedKeysOfAssociativeArray
===file===
<?php
class A {
    const FOO = [
        "bar" => 42
    ];
    /** @return key-of<A::FOO> */
    public function getKey() {
        return "adams";
    }
}

===expect===
InvalidReturnStatement
===ignore===
TODO
