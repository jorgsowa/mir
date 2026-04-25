===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @psalm-type Payload = User
 * @phpstan-type AlternatePayload = User
 * @method Payload get()
 * @method AlternatePayload getAlternate()
 */
class Repository {}

function test(Repository $repo): void {
    $repo->get()->name();
    $repo->getAlternate()->name();
    $repo->get()->missing();
}
===expect===
UndefinedMethod: Method User::missing() does not exist
