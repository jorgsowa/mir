===description===
rawObjectIteration
===file===
<?php
                    class A {
                        /** @var ?string */
                        public $foo;
                    }
                    function example() : Generator {
                        $arr = new A;

                        yield from $arr;
                    }
===expect===
RawObjectIteration
===ignore===
TODO
