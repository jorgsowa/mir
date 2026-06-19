===description===
Only int allowed for key of list
===file===
<?php
class A {
    /**
     * @return key-of<list<string>>
     */
    public function getKey() {
        return "42";
    }
}

===expect===
InvalidReturnType@7:8-7:20: Return type '"42"' is not compatible with declared 'int'
