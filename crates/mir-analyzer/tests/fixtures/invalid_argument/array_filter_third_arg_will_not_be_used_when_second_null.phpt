===description===
arrayFilterThirdArgWillNotBeUsedWhenSecondNull
===file===
<?php
                    array_filter( $arg, null, ARRAY_FILTER_USE_BOTH );
===expect===
InvalidArgument
===ignore===
TODO
