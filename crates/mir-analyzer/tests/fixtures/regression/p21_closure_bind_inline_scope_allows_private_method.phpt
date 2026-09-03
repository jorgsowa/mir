===description===
FP-P21: a closure literal passed directly as `Closure::bind`'s first argument,
rebound to a class-name-literal `$newScope`, must be checked against that
scope for private-method visibility — not its lexically enclosing class.
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
        return Closure::bind(function ($list) use ($copier) {
            return $copier->recursiveCopy($list);
        }, $copier, DeepCopy::class);
    }
}
===expect===
