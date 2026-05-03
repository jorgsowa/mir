===description===
possiblyInvalidConcat
===file===
<?php
                    $b = rand(0, 1) ? [] : "hello";
                    echo $b . "goodbye";
===expect===
PossiblyInvalidOperand
===ignore===
TODO
