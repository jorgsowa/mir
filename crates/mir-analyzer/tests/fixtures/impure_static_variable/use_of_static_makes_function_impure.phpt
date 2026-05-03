===description===
useOfStaticMakesFunctionImpure
===file===
<?php
                    /** @psalm-pure */
                    function addCumulative(int $left) : int {
                        /** @var int */
                        static $i = 0;
                        $i += $left;
                        return $left;
                    }
===expect===
ImpureStaticVariable
===ignore===
TODO
