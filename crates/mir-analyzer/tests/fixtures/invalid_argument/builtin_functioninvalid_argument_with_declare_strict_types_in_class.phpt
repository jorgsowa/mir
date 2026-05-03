===description===
builtinFunctioninvalidArgumentWithDeclareStrictTypesInClass
===file===
<?php declare(strict_types=1);
                    class A {
                        public function foo() : void {
                            $s = substr(5, 4);
                        }
                    }
===expect===
InvalidArgument
===ignore===
TODO
