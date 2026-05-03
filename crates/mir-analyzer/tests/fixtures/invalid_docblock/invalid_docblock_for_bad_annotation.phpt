===description===
invalidDocblockForBadAnnotation
===file===
<?php
                    /**
                     * @param-out array<a(),bool> $ar
                     */
                    function foo(array &$ar) : void {}
===expect===
InvalidDocblock
===ignore===
TODO
