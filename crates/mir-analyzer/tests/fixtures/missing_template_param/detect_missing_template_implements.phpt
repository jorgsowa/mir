===description===
detectMissingTemplateImplements
===file===
<?php
                    /** @template T */
                    interface A {}
                    final class B implements A {}
                
===expect===
MissingTemplateParam
===ignore===
TODO
