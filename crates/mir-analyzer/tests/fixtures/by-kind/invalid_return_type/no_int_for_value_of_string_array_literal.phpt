===description===
No int for value of string array literal
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
InvalidReturnType@7:8-7:18: Return type '42' is not compatible with declared 'string'
