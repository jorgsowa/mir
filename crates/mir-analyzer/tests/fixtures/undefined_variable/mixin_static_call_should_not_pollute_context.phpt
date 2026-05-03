===description===
mixinStaticCallShouldNotPolluteContext
===file===
<?php
                    /**
                     * @template T
                     */
                    class Foo
                    {
                        public function foobar(): void {}
                    }

                    /**
                     * @template T
                     * @mixin Foo<T>
                     */
                    class Bar
                    {
                        public function baz(): self
                        {
                            self::foobar();
                            return $__tmp_mixin_var__;
                        }
                    }
===expect===
UndefinedVariable
===ignore===
TODO
