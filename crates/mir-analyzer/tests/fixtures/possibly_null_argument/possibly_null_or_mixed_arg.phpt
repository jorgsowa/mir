===description===
possiblyNullOrMixedArg
===file===
<?php
                    class A {
                        /**
                         * @var mixed
                         */
                        public $foo;
                    }

                    function takesString(string $s) : void {}

                    function takesA(?A $a) : void {
                        /**
                         * @psalm-suppress PossiblyNullPropertyFetch
                         * @psalm-suppress MixedArgument
                         */
                        takesString($a->foo);
                    }
===expect===
PossiblyNullArgument
===ignore===
TODO
