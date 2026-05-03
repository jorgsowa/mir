===description===
thisInStaticArrowFunction
===file===
<?php
                    class C {
                        public int $a = 1;
                        public function f(): int {
                            $f = static fn(): int => $this->a;
                            return $f();;
                        }
                    }
                
===expect===
InvalidScope
===ignore===
TODO
