===description===
Only defined keys of associative array
===file===
<?php
class A {
    const FOO = [
        "bar" => 42
    ];
    /** @return key-of<A::FOO> */
    public function getKey() {
        return "adams";
//      ^^^^^^^^^^^^^^^ InvalidReturnType: Return type '"adams"' is not compatible with declared 'key-of<A::FOO>'
    }
}
