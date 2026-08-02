===description===
FP-P21: the assign-then-rebind idiom also applies to the instance form
`$closure->bindTo($newThis, $newScope)`, not just the static `Closure::bind`.
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
        return $copy->bindTo(null, DeepCopy::class);
    }
}
===expect===
