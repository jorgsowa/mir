===description===
suppressUnusedSuppressionByItselfIsNotSuppressed
===file===
<?php
                    class Foo {
                        /**
                         * @psalm-suppress UnusedPsalmSuppress
                         */
                        public string $bar = "baz";
                    }
                
===expect===
UnusedPsalmSuppress
===ignore===
TODO
