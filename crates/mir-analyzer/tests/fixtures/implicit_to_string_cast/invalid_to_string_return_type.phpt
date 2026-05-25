===description===
Invalid to string return type
===file===
<?php
class A {
    function __toString(): void { }
}
===expect===
InvalidToString
===ignore===
TODO
