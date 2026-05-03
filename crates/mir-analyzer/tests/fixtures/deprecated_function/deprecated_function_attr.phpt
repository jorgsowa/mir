===description===
deprecatedFunctionAttr
===file===
<?php
                    #[Deprecated]
                    function a(): void {}
                    a();
                
===expect===
DeprecatedFunction
===ignore===
TODO
