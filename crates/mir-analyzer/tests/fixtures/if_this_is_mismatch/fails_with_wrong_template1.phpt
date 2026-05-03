===description===
failsWithWrongTemplate1
===file===
<?php

                /**
                 * @template T
                 */
                class a {
                    /**
                     * @var T
                     */
                    private $data;
                    /**
                     * @param T $data
                     */
                    public function __construct($data) {
                        $this->data = $data;
                    }
                    /**
                     * @psalm-if-this-is a<int>
                     */
                    public function test(): void {
                    }
                }

                $i = new a("test");
                $i->test();
                
===expect===
IfThisIsMismatch
===ignore===
TODO
