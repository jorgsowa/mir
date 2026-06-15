===description===
Invalid inferred to string return type
===file===
<?php
class A {
    function __toString() { }
}
===expect===
InvalidToString@3:26-3:29: Method A::__toString() must return a string
