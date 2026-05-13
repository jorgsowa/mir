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
UnusedVariable@4:28: Variable $s is never read
InvalidArgument@4:40: Argument $string of substr() expects 'string', got '5'
