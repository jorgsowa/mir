===description===
A docblock written after a method's attributes belongs to that method, not to the next one.
===file===
<?php
final class Api {
    #[\NoDiscard]
    /** @deprecated */
    public function old(): int { return 1; }
    public function current(): void {}
}
$api = new Api();
$api->old();
//<^^^^^^^^^^^ DeprecatedMethod: Method Api::old() is deprecated
$api->current();
