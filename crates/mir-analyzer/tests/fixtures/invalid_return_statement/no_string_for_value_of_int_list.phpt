===description===
noStringForValueOfIntList
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
InvalidReturnStatement
===ignore===
TODO
