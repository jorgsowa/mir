===description===
Companion to call_magic_honors_declared_return_type: without its own @return
docblock, __call's dispatch result must still collapse to mixed, so a chained
call off it is correctly flagged. Guards against over-widening the fix to
always trust __call's presence regardless of its declared type.
===file===
<?php
class TestDouble {
    public function __call(string $name, array $arguments): mixed {
        return null;
    }
}

function test(): void {
    (new TestDouble())->anyMethod()->anotherMethod();
}
===expect===
MixedMethodCall@9:4-9:52: Method anotherMethod() called on mixed type
