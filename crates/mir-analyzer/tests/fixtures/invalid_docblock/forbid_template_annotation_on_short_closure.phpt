===description===
forbidTemplateAnnotationOnShortClosure
===file===
<?php
                    /** @template T */
                    fn(): bool => false;
                
===expect===
InvalidDocblock
===ignore===
TODO
