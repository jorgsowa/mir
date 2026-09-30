===description===
Builtin functioninvalid argument with declare strict types in class
===file===
<?php declare(strict_types=1);
                    class A {
                        public function foo() : void {
                            $s = substr(5, 4);
//                          ^^ UnusedVariable: Variable $s is never read
//                                      ^ InvalidArgument: Argument $string of substr() expects 'string', got '5'
                        }
                    }
===expect===
