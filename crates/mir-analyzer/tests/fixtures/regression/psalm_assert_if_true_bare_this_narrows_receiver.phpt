===description===
M17: `@psalm-assert-if-true`/`@phpstan-assert-if-true Type $this` (bare
receiver, no `->prop`) narrows the receiver itself to the asserted type —
sibling of the already-working `$this->prop` form. An unguarded call still
correctly flags, since nothing proves the narrower type without the guard.
===config===
suppress=UnusedParam
===file===
<?php
class TestData {
    /** @phpstan-assert-if-true DataFromDataProvider $this */
    public function isFromDataProvider(): bool { return false; }
}
final class DataFromDataProvider extends TestData {}

function needsSpecific(DataFromDataProvider $d): void {}

function guarded(TestData $data): void {
    if ($data->isFromDataProvider()) {
        needsSpecific($data);
    }
}

function unguarded(TestData $data): void {
    needsSpecific($data);
}
===expect===
ArgumentTypeCoercion@17:18-17:23: Argument $d of needsSpecific() expects 'DataFromDataProvider', got 'TestData' — coercion may fail at runtime
