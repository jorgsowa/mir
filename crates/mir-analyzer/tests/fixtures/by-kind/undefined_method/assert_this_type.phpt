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
//          ^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Exception is thrown but not declared in @throws
        }

        return true;
    }
}

class FooType extends Type {
    public function bar(): void {}
}

function takesType(Type $t) : void {
    $t->bar();
//  ^^^^^^^^^ UndefinedMethod: Method Type::bar() does not exist
    $t->isFoo();
}
===expect===
