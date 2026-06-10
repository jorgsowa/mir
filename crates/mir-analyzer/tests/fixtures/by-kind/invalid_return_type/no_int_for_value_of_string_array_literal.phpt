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
