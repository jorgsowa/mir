===description===
returnByReferenceNonVariableInShortClosure
===file===
<?php
                    fn &(): int => 45;
                
===expect===
NonVariableReferenceReturn
===ignore===
TODO
