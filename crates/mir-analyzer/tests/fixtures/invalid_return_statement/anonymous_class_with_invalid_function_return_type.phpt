===description===
anonymousClassWithInvalidFunctionReturnType
===file===
<?php
                    $foo = new class {
                        public function a(): string {
                            return 5;
                        }
                    };
===expect===
InvalidReturnStatement
===ignore===
TODO
