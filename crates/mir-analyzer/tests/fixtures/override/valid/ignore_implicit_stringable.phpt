===description===
Ignore implicit stringable
===file===
                    <?php
                    class A {
                        public function __toString(): string {
                            return "";
                        }
                    }
                
===expect===

