===description===
arrayFilterKeyCallback
===file===
<?php
                    /**
                     * @var array<int, string> $arg
                     */
                    array_filter($arg, "strlen", ARRAY_FILTER_USE_KEY);
===expect===
InvalidScalarArgument
===ignore===
TODO
