===description===
invalidExtendsAnnotatedFinalClass
===file===
<?php

                /**
                * @final
                */
                class DoctrineA {}

                class DoctrineB extends DoctrineA {}'

===expect===
InvalidExtendClass
===ignore===
TODO
