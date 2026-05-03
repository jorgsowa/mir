===description===
classConstantNoClass
===file===
<?php
                    namespace Ns;

                    /** @psalm-param "foo"|"bar"|C::A|C::B $s */
                    function foo($s) : void {}
===expect===
UndefinedDocblockClass
===ignore===
TODO
