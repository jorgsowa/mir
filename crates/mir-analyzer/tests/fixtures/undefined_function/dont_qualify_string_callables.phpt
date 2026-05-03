===description===
dontQualifyStringCallables
===file===
<?php
                    namespace NS;

                    function ff() : void {}

                    function run(callable $f) : void {
                        $f();
                    }

                    run("ff");
===expect===
UndefinedFunction
===ignore===
TODO
