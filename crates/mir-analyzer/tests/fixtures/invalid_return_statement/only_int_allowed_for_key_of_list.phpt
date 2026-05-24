===description===
onlyIntAllowedForKeyOfList
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
InvalidReturnStatement
===ignore===
TODO
