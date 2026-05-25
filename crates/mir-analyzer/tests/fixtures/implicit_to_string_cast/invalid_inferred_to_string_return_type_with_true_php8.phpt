===description===
invalidInferredToStringReturnTypeWithTruePhp8
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
