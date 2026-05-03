===description===
attributeClassHasNoAttributeAnnotation
===file===
<?php
                    class A {}

                    #[A]
                    class B {}
===expect===
InvalidAttribute
===ignore===
TODO
