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
    /**
     * @param Iterator<TKey, TValue> $i
     */
    public function __construct(Iterator $i) {
        parent::__construct($i);
    }
}
===expect===
UnimplementedAbstractMethod@7:0-7:45: Class DedupeIterator must implement abstract method accept()
