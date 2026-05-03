===description===
missingReturnType
===file===
<?php
                    interface foo {
                        public function withoutAnyReturnType();
                    }
===expect===
MissingReturnType
===ignore===
TODO
