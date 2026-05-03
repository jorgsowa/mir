===description===
magicParentCallShouldNotPolluteContext
===file===
<?php
                    /**
                     * @method baz(): Foo
                     */
                    class Foo
                    {
                        public function __call()
                        {
                            return new self();
                        }
                    }

                    class Bar extends Foo
                    {
                        public function baz(): Foo
                        {
                            parent::baz();
                            return $__tmp_parent_var__;
                        }
                    }
===expect===
UndefinedVariable
===ignore===
TODO
