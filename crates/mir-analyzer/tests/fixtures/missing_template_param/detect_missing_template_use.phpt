===description===
detectMissingTemplateUse
===file===
<?php
                    /** @template T */
                    trait A {}
                    final class B {
                        use A;
                    }
                
===expect===
MissingTemplateParam
===ignore===
TODO
