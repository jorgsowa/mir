===description===
preventExtendingInterface
===file===
<?php
                    interface Foo {}

                    class Bar extends Foo {}
===expect===
UndefinedClass
===ignore===
TODO
