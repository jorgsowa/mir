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
    }
}

===expect===
InvalidReturnType@7:8-7:21: Return type '"foo"' is not compatible with declared 'int'
