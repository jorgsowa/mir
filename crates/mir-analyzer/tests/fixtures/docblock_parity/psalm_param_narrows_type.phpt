===description===
psalm param narrows type
===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @param mixed $value
 * @psalm-param User $value
 */
function process($value): void {
    $value->name();
    $value->missing();
}
===expect===
UndefinedMethod@12:5: Method User::missing() does not exist
