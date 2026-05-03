===description===
arrayFilterThirdArgInvalidBehavesLike0
===file===
<?php
                    array_filter( $arg, "strlen", 3 );
===expect===
PossiblyInvalidArgument
===ignore===
TODO
