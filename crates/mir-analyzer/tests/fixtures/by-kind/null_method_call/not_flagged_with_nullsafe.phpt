===description===
NullMethodCall and PossiblyNullMethodCall do NOT fire when using the nullsafe
operator (?->). The nullsafe operator is designed to handle null receivers.
===file===
<?php
class Foo { public function bar(): void {} }
function test(?Foo $obj): void {
    $obj?->bar();
}
===expect===
