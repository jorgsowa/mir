===description===
arrayFilterBothCallback
===file===
<?php
                    /**
                     * @var array<string, float> $arg
                     */
                    array_filter($arg, "strlen", ARRAY_FILTER_USE_BOTH);
===expect===
InvalidArgument
===ignore===
TODO
