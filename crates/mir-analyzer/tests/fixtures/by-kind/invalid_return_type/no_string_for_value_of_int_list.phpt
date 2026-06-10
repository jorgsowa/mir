===description===
No string for value of int list
===ignore===
TODO
===file===
<?php
class A {
    /**
     * @return value-of<list<int>>
     */
    public function getValue() {
        return "42";
    }
}

===expect===
