===description===
attributeClassCannotHavePrivateConstructor
===file===
<?php
                    #[Attribute]
                    class Baz {
                        private function __construct() {}
                    }
===expect===
InvalidAttribute
===ignore===
TODO
