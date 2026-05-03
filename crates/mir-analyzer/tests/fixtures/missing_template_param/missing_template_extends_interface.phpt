===description===
missingTemplateExtendsInterface
===file===
<?php
                    /** @template T */
                    interface A {}
                    interface B extends A {}
                
===expect===
MissingTemplateParam
===ignore===
TODO
