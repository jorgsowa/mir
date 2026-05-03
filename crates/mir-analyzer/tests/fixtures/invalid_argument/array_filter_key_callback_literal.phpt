===description===
arrayFilterKeyCallbackLiteral
===file===
<?php
                    array_filter(["a" => 5, "b" => 12, "c" => null], "abs", ARRAY_FILTER_USE_KEY);
===expect===
InvalidArgument
===ignore===
TODO
