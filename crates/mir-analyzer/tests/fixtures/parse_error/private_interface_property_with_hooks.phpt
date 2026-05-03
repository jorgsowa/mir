===description===
privateInterfacePropertyWithHooks
===file===
<?php
                    interface A {
                        private string $value { get; }
                    }
===expect===
ParseError
===ignore===
TODO
