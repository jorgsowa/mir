===description===
useDuplicateName
===file===
<?php
                    $foo = "bar";

                    $a = function (string $foo) use ($foo) : string {
                      return $foo;
                    };
===expect===
DuplicateParam
===ignore===
TODO
