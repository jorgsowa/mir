===description===
arrayMapWithNonCallableStringArray
===file===
<?php
                    $foo = ["one", "two"];
                    array_map($foo, ["hello"]);
===expect===
InvalidArgument
===ignore===
TODO
