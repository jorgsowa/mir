===description===
invalidExtendsFinalClassAndOtherAnnotation
===file===
<?php

                /**
                * @something-else-no-final annotation
                */
                final class DoctrineA {}

                class DoctrineB extends DoctrineA {}'

===expect===
InvalidExtendClass
===ignore===
TODO
