===description===
unusedVarWithConditionalAddition
===file===
<?php
                    $a = 5;
                    if (rand(0, 1)) {
                        $a = $a + 1;
                    }
===expect===
UnusedVariable
===ignore===
TODO
