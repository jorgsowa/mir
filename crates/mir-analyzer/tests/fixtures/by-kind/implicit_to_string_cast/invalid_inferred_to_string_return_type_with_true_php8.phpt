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
InvalidToString@3:27-6:28: Method A::__toString() must return a string
