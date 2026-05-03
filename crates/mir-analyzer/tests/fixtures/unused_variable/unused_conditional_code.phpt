===description===
unusedConditionalCode
===file===
<?php
                    $a = 5;
                    if (rand(0, 1)) {
                      $a = $a + 5;
                    }
===expect===
UnusedVariable
===ignore===
TODO
