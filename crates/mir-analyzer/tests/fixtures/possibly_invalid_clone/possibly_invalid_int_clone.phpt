===description===
possiblyInvalidIntClone
===file===
<?php
                    $a = rand(0, 1) ? 5 : new Exception();
                    clone $a;
===expect===
PossiblyInvalidClone
===ignore===
TODO
