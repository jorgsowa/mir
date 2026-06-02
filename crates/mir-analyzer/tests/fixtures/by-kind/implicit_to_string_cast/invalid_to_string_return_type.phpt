===description===
Invalid to string return type
===file===
<?php
class A {
    function __toString(): void { }
}
===expect===
InvalidToString@3:33-3:36: Method A::__toString() must return a string
