===description===
mixedPropertyFetch
===file===
<?php
                    class Foo {
                        /** @var string */
                        public $foo = "";
                    }

                    /** @var mixed */
                    $a = (new Foo());

                    echo $a->foo;
===expect===
MixedPropertyFetch
===ignore===
TODO
