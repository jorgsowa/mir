===description===
Invalid inferred to string return type
===file===
<?php
class A {
    function __toString() { }
//                        ^^^ InvalidToString: Method A::__toString() must return a string
}
===expect===
