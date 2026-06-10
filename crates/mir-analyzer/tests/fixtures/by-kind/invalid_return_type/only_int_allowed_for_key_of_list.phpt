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
