===description===
FP-P21: `$newScope` written as a bare string literal (`'DeepCopy'`) rather
than `DeepCopy::class` must resolve the same way — both are the syntactic
literal forms real code uses.
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
        }, $copier, 'DeepCopy');
    }
}
===expect===
