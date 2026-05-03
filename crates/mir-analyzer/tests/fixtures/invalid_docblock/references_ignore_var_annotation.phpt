===description===
referencesIgnoreVarAnnotation
===file===
<?php
                    $a = 1;
                    /** @var int */
                    $b = &$a;
                
===expect===
InvalidDocblock
===ignore===
TODO
