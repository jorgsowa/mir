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
InvalidToString@3:26-6:5: Method A::__toString() must return a string
UnusedSuppress@5:0-5:0: Suppress annotation for 'InvalidReturnStatement' is never used
