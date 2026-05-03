===description===
sensitiveParameterOnMethod
===file===
<?php

                    namespace SensitiveParameter;

                    use SensitiveParameter;

                    class HelloWorld {
                        #[SensitiveParameter]
                        public function __construct(
                            string $password
                        ) {}
                    }
                
===expect===
Attribute SensitiveParameter cannot be used on a method
===ignore===
TODO
