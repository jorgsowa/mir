===description===
arrayMapWithNonCallableIntArray
===file===
<?php
                    $foo = [1, 2];
                    array_map($foo, ["hello"]);
===expect===
InvalidArgument
===ignore===
TODO
