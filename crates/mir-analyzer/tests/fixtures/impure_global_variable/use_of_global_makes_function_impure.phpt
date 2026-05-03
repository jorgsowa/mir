===description===
useOfGlobalMakesFunctionImpure
===file===
<?php
                    /** @psalm-pure */
                    function addCumulative(int $left) : int {
                        /** @var int */
                        global $i;
                        $i ??= 0;
                        $i += $left;
                        return $left;
                    }
===expect===
ImpureGlobalVariable
===ignore===
TODO
