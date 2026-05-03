===description===
invalidInferredToStringReturnType
===file===
<?php
                    class A {
                        function __toString() { }
                    }
===expect===
InvalidToString
===ignore===
TODO
