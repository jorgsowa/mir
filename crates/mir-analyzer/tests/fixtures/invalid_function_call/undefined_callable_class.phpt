===description===
undefinedCallableClass
===file===
<?php
                    class A {
                        public function getFoo(): Foo
                        {
                            return new Foo([]);
                        }

                        /**
                         * @param  mixed $argOne
                         * @param  mixed $argTwo
                         * @return void
                         */
                        public function bar($argOne, $argTwo)
                        {
                            $this->getFoo()($argOne, $argTwo);
                        }
                    }
===expect===
InvalidFunctionCall
===ignore===
TODO
