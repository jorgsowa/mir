===description===
Invalid inferred to string return type with true php8
===file===
<?php
class A {
    function __toString() {
//                        ^ +3:5 InvalidToString: Method A::__toString() must return a string
        /** @suppress InvalidReturnStatement */
//                    ^^^^^^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'InvalidReturnStatement' is never used
        return true;
    }
}
===expect===
