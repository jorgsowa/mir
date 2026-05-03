===description===
abstractReflectedClassMethod
===file===
<?php
                    /**
                     * @template TKey
                     * @template TValue
                     * @extends FilterIterator<TKey, TValue, Iterator<TKey, TValue>>
                     */
                    class DedupeIterator extends FilterIterator {
                        /**
                         * @param Iterator<TKey, TValue> $i
                         */
                        public function __construct(Iterator $i) {
                            parent::__construct($i);
                        }
                    }
===expect===
UnimplementedAbstractMethod
===ignore===
TODO
