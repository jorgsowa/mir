===description===
invalidMixedClone
===file===
<?php
                    /** @var mixed $a */
                    $a = 5;
                    clone $a;
===expect===
MixedClone@4:20: cannot clone mixed
