===description===
FALSE POSITIVE reproducer. A __call magic-dispatch method's own declared
@return type (a fluent test-double stub returning `static`) should be honored
instead of collapsing to mixed, so a chained call off its result isn't flagged
MixedMethodCall.
===file===
<?php
class TestDouble {
    /** @return static */
    public function __call(string $name, array $arguments): static {
        return $this;
    }
}

function test(): void {
    (new TestDouble())->anyMethod()->anotherMethod();
}
===expect===
