===description===
attributeInvalidTargetMethod
===file===
<?php
                    class Foo {
                        #[Attribute]
                        public function bar(): void {}
                    }
                
===expect===
InvalidAttribute
===ignore===
TODO
