===description===
Invalid inferred to string return type with true php8
===file===
<?php
class A {
    function __toString() {
        /** @suppress InvalidReturnStatement */
        return true;
    }
}
===expect===
InvalidToString
===ignore===
TODO
