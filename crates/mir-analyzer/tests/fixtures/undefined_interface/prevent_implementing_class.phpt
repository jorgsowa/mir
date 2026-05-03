===description===
preventImplementingClass
===file===
<?php
                    class Foo {}

                    class Bar implements Foo {}
===expect===
UndefinedInterface
===ignore===
TODO
