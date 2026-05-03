===description===
missingParamType
===file===
<?php
                    interface foo {
                        public function withoutAnyReturnType($s) : void;
                    }
===expect===
MissingParamType
===ignore===
TODO
