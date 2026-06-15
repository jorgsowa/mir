===description===
Builtin functioninvalid argument with declare strict types in class
===file===
<?php declare(strict_types=1);
                    class A {
                        public function foo() : void {
                            $s = substr(5, 4);
                        }
                    }
===expect===
UnusedVariable@4:28-4:30: Variable $s is never read
InvalidArgument@4:40-4:41: Argument $string of substr() expects 'string', got '5'
