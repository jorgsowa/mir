===description===
attributeInvalidTargetProperty
===file===
<?php
                    class Foo {
                        #[Attribute]
                        public string $bar = "baz";
                    }
                
===expect===
InvalidAttribute
===ignore===
TODO
