===description===
Key of array literal
===ignore===
TODO
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
