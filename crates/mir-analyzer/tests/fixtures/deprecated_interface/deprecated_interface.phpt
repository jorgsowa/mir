===description===
deprecatedInterface
===file===
<?php
                    /** @deprecated */
                    interface Container {}

                    class A implements Container {}
===expect===
DeprecatedInterface
===ignore===
TODO
