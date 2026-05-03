===description===
classRedefinitionInNamespace
===file===
<?php
                    namespace Aye {
                        class Foo {}
                        class Foo {}
                    }
===expect===
DuplicateClass
===ignore===
TODO
