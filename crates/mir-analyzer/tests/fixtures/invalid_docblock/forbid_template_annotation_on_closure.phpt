===description===
forbidTemplateAnnotationOnClosure
===file===
<?php
                    /** @template T */
                    function (): void {};
                
===expect===
InvalidDocblock
===ignore===
TODO
