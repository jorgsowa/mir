===description===
invalidInferredToStringReturnTypeWithTruePhp8
===file===
<?php
                    class A {
                        function __toString() {
                            /** @psalm-suppress InvalidReturnStatement */
                            return true;
                        }
                    }
===expect===
InvalidToString
===ignore===
TODO
