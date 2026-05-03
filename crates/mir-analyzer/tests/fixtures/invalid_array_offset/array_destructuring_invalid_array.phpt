===description===
arrayDestructuringInvalidArray
===file===
<?php
                    $a = 42;

                    [$id2, $name2] = $a;
===expect===
InvalidArrayOffset
===ignore===
TODO
