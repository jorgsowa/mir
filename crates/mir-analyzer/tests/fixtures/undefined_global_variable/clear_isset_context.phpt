===description===
clearIssetContext
===file===
<?php
                    function greet(bool $arg): ?string
                    {
                        return $arg ? "hi" : null;
                    }

                    echo greet($undef) ?? "bye";
===expect===
UndefinedGlobalVariable
===ignore===
TODO
