===description===
abstractClassCannotBeAttributeClass
===file===
<?php
                    #[Attribute]
                    abstract class Baz {}
===expect===
InvalidAttribute
===ignore===
TODO
