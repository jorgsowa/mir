===description===
Invalid to string return type
===file===
<?php
class A {
    function __toString(): void { }
}
===expect===
InvalidToString@3:32-3:35: Method A::__toString() must return a string
