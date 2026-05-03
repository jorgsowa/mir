===description===
undefinedConstantInAttribute
===file===
<?php
                    #[Attribute]
                    class Foo
                    {
                        public function __construct(int $i) {}
                    }

                    #[Foo(self::BAR_CONST)]
                    class Bar {}
                
===expect===
UndefinedConstant
===ignore===
TODO
