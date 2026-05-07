===description===
invalidMixedClone
===file===
<?php
                    /** @var mixed $a */
                    $a = 5;
                    clone $a;
===expect===
MixedClone@7:2: cannot clone mixed
