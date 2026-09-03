===description===
FP-P21: the common "assign a closure to a variable, then immediately rebind
it" idiom (as seen in myclabs/deep-copy's SplDoublyLinkedListFilter) must also
be checked against the rebound scope, not the closure's lexically enclosing
class. Only the statement immediately following the assignment is trusted for
this — nothing reassigns `$copy` in between.
===config===
suppress=MissingClosureReturnType
===file===
<?php
class DeepCopy {
    private function recursiveCopy($x) {
        return $x;
    }
}
class Filter {
    public function make(DeepCopy $copier) {
        $copy = function ($list) use ($copier) {
            return $copier->recursiveCopy($list);
        };
        return Closure::bind($copy, null, DeepCopy::class);
    }
}
===expect===
