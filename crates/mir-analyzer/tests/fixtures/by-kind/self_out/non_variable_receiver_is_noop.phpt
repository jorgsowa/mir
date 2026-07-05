===description===
@psalm-self-out on a call whose receiver isn't a simple variable (e.g. a
chained call result) is a no-op — there is no variable to retype, and this
must not crash the analyzer.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    /** @psalm-self-out Foo */
    public function touch(): void {}

    public function self(): Foo {
        return $this;
    }
}

function make(): Foo {
    return new Foo();
}

make()->touch();
(new Foo())->self()->touch();

===expect===
