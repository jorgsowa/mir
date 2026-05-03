===description===
classRedefinitionInSeparateNamespace
===file===
<?php
                    namespace Aye {
                        class Foo {}
                    }
                    namespace Aye {
                        class Foo {}
                    }
===expect===
DuplicateClass
===ignore===
TODO
