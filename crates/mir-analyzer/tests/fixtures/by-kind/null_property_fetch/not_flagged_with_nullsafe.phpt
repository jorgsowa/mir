===description===
NullPropertyFetch and PossiblyNullPropertyFetch do NOT fire when using the
nullsafe operator (?->). The nullsafe operator is designed to handle null
receivers.
===file===
<?php
class Obj { public string $name = 'x'; }
function test(?Obj $obj): string {
    return $obj?->name ?? '';
}
===expect===
