===description===
getTypeArgWrongArgs
===file===
<?php
                    function testInt(int $var): void {

                    }

                    function testString(string $var): void {

                    }

                    $a = rand(0, 10) ? 1 : "two";

                    switch (gettype($a)) {
                        case "string":
                            testInt($a);

                        case "integer":
                            testString($a);
                    }
===expect===
InvalidScalarArgument
===ignore===
TODO
