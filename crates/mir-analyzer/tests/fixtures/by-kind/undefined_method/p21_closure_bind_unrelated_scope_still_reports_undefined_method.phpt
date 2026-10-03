===description===
FP-P21 control: rebinding to a class-name literal that is NOT the receiver's
own class must still report UndefinedMethod — the scope-override fix must
not blanket-suppress private-method visibility checks inside a rebound
closure, only apply the actually-resolved scope.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class DeepCopy {
    private function recursiveCopy($x) {
        return $x;
    }
}
class Other {
}
class Filter {
    public function make(DeepCopy $copier) {
        $copy = function ($list) use ($copier) {
            return $copier->recursiveCopy($list);
//                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method DeepCopy::recursiveCopy() does not exist
        };
        return Closure::bind($copy, null, Other::class);
    }
}
===expect===
