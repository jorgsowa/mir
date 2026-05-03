===description===
invalidIterableArg
===file===
<?php
                    /**
                     * @param  iterable<string> $iter
                     */
                    function iterator(iterable $iter): void
                    {
                        foreach ($iter as $val) {
                            //
                        }
                    }

                    class A {
                    }

                    iterator(new A());
===expect===
InvalidArgument
===ignore===
TODO
