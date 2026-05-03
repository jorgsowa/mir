===description===
invalidExtendsFinalClass
===file===
<?php

                final class A {}

                class B extends A {}'

===expect===
InvalidExtendClass
===ignore===
TODO
