===description===
noParent
===file===
<?php
                    class C {
                        #[Override]
                        public function f(): void {}
                    }
                
===expect===
InvalidOverride
===ignore===
TODO
