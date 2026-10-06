===description===
Key of array literal
===file===
<?php
class A {
    /**
     * @return key-of<array<int, string>>
     */
    public function getKey() {
        return "foo";
//      ^^^^^^^^^^^^^ InvalidReturnType: Return type '"foo"' is not compatible with declared 'int'
    }
}
