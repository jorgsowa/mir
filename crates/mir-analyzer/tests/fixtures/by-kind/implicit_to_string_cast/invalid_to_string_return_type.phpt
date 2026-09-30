===description===
Invalid to string return type
===file===
<?php
class A {
    function __toString(): void { }
//                              ^^^ InvalidToString: Method A::__toString() must return a string
}
===expect===
