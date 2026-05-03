===description===
nullablePropertyAssignment
===file===
<?php
                    $a = null;

                    $a->foo = "hello";
===expect===
NullPropertyAssignment
===ignore===
TODO
