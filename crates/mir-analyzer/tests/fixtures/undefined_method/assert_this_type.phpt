===description===
assertThisType
===file===
<?php
                    class Type {
                        /**
                         * @psalm-assert FooType $this
                         */
                        public function isFoo() : bool {
                            if (!$this instanceof FooType) {
                                throw new Exception();
                            }

                            return true;
                        }
                    }

                    class FooType extends Type {
                        public function bar(): void {}
                    }

                    function takesType(Type $t) : void {
                        $t->bar();
                        $t->isFoo();
                    }
===expect===
MissingThrowsDocblock@8:32: Exception Exception is thrown but not declared in @throws
UndefinedMethod@20:24: Method Type::bar() does not exist
