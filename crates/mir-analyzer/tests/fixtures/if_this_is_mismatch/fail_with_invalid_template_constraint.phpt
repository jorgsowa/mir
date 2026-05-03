===description===
failWithInvalidTemplateConstraint
===file===
<?php
                    /** @template T */
                    final class Option { }

                    /**
                     * @template T
                     */
                    final class ArrayList
                    {
                        /**
                         * @template A
                         * @psalm-if-this-is ArrayList<Option<A>>
                         * @return ArrayList<A>
                         */
                        public function compact(): ArrayList
                        {
                            throw new RuntimeException("???");
                        }
                    }

                    /** @var ArrayList<int> $list */
                    $list = new ArrayList();
                    $numbers = $list->compact();
===expect===
IfThisIsMismatch
===ignore===
TODO
