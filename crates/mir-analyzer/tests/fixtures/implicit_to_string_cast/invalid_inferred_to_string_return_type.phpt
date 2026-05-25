===description===
Invalid inferred to string return type
===file===
<?php
class A {
    function __toString() { }
}
===expect===
InvalidToString
===ignore===
TODO
