===description===
Invalid inferred to string return type
===file===
<?php
class A {
    function __toString() { }
}
===expect===
InvalidToString@3:27-3:30: Method A::__toString() must return a string
