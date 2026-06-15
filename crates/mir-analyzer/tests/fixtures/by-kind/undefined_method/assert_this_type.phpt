===description===
Assert this type
===file===
<?php
class Type {
    /**
     * @assert FooType $this
     */
    public function isFoo() : bool {
        if (!$this instanceof FooType) {
            throw new Exception();
        }

        return true;
    }
}

class FooType extends Type {
    public function bar(): void {}
}

function takesType(Type $t) : void {
    $t->bar();
    $t->isFoo();
}
===expect===
MissingThrowsDocblock@8:12-8:34: Exception Exception is thrown but not declared in @throws
UndefinedMethod@20:4-20:13: Method Type::bar() does not exist
