===description===
Abstract reflected class method
===file===
<?php
/**
 * @template TKey
 * @template TValue
 * @extends FilterIterator<TKey, TValue, Iterator<TKey, TValue>>
 */
class DedupeIterator extends FilterIterator {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedAbstractMethod: Class DedupeIterator must implement abstract method accept()
    /**
     * @param Iterator<TKey, TValue> $i
     */
    public function __construct(Iterator $i) {
        parent::__construct($i);
    }
}
===expect===
