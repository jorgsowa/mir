===description===
invalidToStringReturnType
===file===
<?php
                    class A {
                        function __toString(): void { }
                    }
===expect===
InvalidToString
===ignore===
TODO
