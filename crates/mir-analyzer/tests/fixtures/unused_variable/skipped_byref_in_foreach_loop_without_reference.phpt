===description===
SKIPPED-byrefInForeachLoopWithoutReference
===file===
<?php
                    $a = [1, 2, 3];
                    foreach ($a as &$b) {
                        $b = $b + 1;
                    }
===expect===
UnusedVariable
===ignore===
TODO
