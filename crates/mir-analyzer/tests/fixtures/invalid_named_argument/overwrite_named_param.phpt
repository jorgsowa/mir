===description===
overwriteNamedParam
===file===
<?php
                    function test(int $param, int $param2): void {
                        echo $param + $param2;
                    }

                    test(param: 1, param: 2);
===expect===
InvalidNamedArgument
===ignore===
TODO
